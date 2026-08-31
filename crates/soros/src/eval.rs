//! The recursive evaluator.

use crate::program::Program;
use crate::regex::StrCaptures;
use crate::template::Node;

/// Caps how deep `$(...)` calls may recurse before the engine gives up
/// and yields an empty string for that branch.
///
/// Every bundled locale resolves the largest number it can express in a
/// few dozen levels, so this never fires for real data. It exists
/// because a Soros program is arbitrary input (see
/// [`crate::Program::compile`]), and an unbounded recursive evaluator
/// hands a hostile - or merely buggy - rule file a way to exhaust the
/// stack. Rust cannot catch that: a stack overflow aborts the process,
/// taking the caller's program down rather than returning an error they
/// could handle. A depth counter keeps a bad rule file's blast radius
/// inside the one call that triggered it.
pub const MAX_RECURSION_DEPTH: usize = 512;

/// A pair of leading/trailing boundary flags (spec 2.3.2, 2.4.4).
///
/// Two of these travel through the evaluator and mean different things,
/// which is exactly why they are named rather than passed as four loose
/// booleans: `enclosing` is whether the current call is still attached
/// to the original input's leading/trailing boundary, and `position` is
/// whether the node being rendered sits at the start/end of the template
/// it belongs to. A call inherits a boundary only when both agree.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Boundary {
    pub(crate) begin: bool,
    pub(crate) end: bool,
}

impl Boundary {
    /// The outermost call: attached to both boundaries.
    const WHOLE: Self = Self {
        begin: true,
        end: true,
    };

    /// Where `index` sits within a template of `len` nodes.
    fn at(index: usize, len: usize) -> Self {
        Self {
            begin: index == 0,
            end: index + 1 == len,
        }
    }

    /// Narrows a bracket's inner position by the bracket's own: a
    /// `[...]` block is inline and does not start a template of its own,
    /// so its first child is at the start only if the bracket is too.
    fn nested_in(self, outer: Self) -> Self {
        Self {
            begin: self.begin && outer.begin,
            end: self.end && outer.end,
        }
    }
}

impl Program {
    /// Evaluates the program's implicit `$` function against `input`,
    /// treating it as the whole (global) call: it is eligible for both
    /// leading- and trailing-boundary-anchored rules (spec 2.3.2).
    ///
    /// Returns an empty string when no rule matches - the language has
    /// no separate notion of failure.
    ///
    /// ```
    /// // The spec's thousands-separator example.
    /// let program = soros::Program::compile("(\\d+)(\\d{3}) $1,\\2\n(\\d+) \\1\n", "");
    ///
    /// assert_eq!(program.run("1234567"), "1,234,567");
    /// ```
    #[must_use]
    pub fn run(&self, input: &str) -> String {
        self.eval(input, Boundary::WHOLE, 0)
    }

    /// The recursive evaluator behind [`Program::run`].
    ///
    /// A rule whose pattern carried an explicit `^` applies only when
    /// `enclosing.begin` holds, and likewise `$`/`enclosing.end`. The
    /// first rule that matches wins.
    fn eval(&self, input: &str, enclosing: Boundary, depth: usize) -> String {
        if depth > MAX_RECURSION_DEPTH {
            return String::new();
        }

        for rule in &self.rules {
            if (rule.begin && !enclosing.begin) || (rule.end && !enclosing.end) {
                continue;
            }

            if let Some(caps) = rule.re.captures(input) {
                return self.render(&rule.tmpl, &caps, enclosing, depth);
            }
        }

        String::new()
    }

    /// Evaluates a parsed replacement template against the capture
    /// groups of the rule that matched, recursing into the program for
    /// every `$(...)`/`$N` call it contains.
    fn render(
        &self,
        tmpl: &[Node],
        caps: &StrCaptures<'_>,
        enclosing: Boundary,
        depth: usize,
    ) -> String {
        let mut out = String::new();

        for (i, node) in tmpl.iter().enumerate() {
            let position = Boundary::at(i, tmpl.len());
            out.push_str(&self.render_node(node, caps, enclosing, position, depth));
        }

        out
    }

    /// Evaluates a single template node.
    ///
    /// `position` says whether this node sits at the very start/end of
    /// the template it belongs to, which - absent an explicit `|`
    /// boundary modifier - is how a call inherits `enclosing`
    /// (spec 2.4.4).
    fn render_node(
        &self,
        node: &Node,
        caps: &StrCaptures<'_>,
        enclosing: Boundary,
        position: Boundary,
        depth: usize,
    ) -> String {
        match node {
            Node::Literal(text) => text.clone(),
            Node::Backref(group) => group_text(caps, *group).to_owned(),
            Node::Call {
                arg,
                pipe_before,
                pipe_after,
            } => {
                let built = self.render(arg, caps, enclosing, depth);
                let inherited = Boundary {
                    begin: *pipe_before || (position.begin && enclosing.begin),
                    end: *pipe_after || (position.end && enclosing.end),
                };

                self.eval(&built, inherited, depth + 1)
            }
            Node::Optional(children) => {
                self.render_optional(children, caps, enclosing, position, depth)
            }
            // Folded into the neighboring calls' flags at parse time and
            // dropped there; never reaches the evaluator.
            Node::Pipe => String::new(),
        }
    }

    /// Implements conditional text (spec 2.4.5): the rendered content of
    /// a `[...]` block is kept only if the **first** call inside it
    /// evaluated to a non-empty string. Later calls are ordinary
    /// content - they may be empty without discarding the block.
    ///
    /// That "first call" rule is not an arbitrary reading. The reference
    /// implementations do not evaluate brackets at all; they rewrite
    /// `[before$CALL after]` into `$(SEP before SEP $CALL SEP after)`
    /// with a text substitution, and let two hidden rules decide:
    /// `SEP(.*)SEP(.+)SEP(.*)` keeps the block, `SEP.*SEPSEP.*` drops
    /// it. Only one call ever lands in that middle slot - the first -
    /// so only the first can decide.
    ///
    /// Requiring *every* call to be non-empty looks like the same rule
    /// and is not. `lg.sor` has
    /// `1(\d{6}) kakadde [ka$(:1) ne $1]`, where the second call is
    /// empty for a round million: under "every call" the block vanishes
    /// and `1000000` loses its ending.
    fn render_optional(
        &self,
        children: &[Node],
        caps: &StrCaptures<'_>,
        enclosing: Boundary,
        outer: Boundary,
        depth: usize,
    ) -> String {
        let mut out = String::new();
        let mut keep = true;
        let mut decided = false;

        for (i, child) in children.iter().enumerate() {
            let position = Boundary::at(i, children.len()).nested_in(outer);
            let rendered = self.render_node(child, caps, enclosing, position, depth);

            if !decided && matches!(child, Node::Call { .. }) {
                keep = !rendered.is_empty();
                decided = true;
            }

            out.push_str(&rendered);
        }

        if keep { out } else { String::new() }
    }
}

/// Returns capture group `i` (0 being the whole match), or `""` when the
/// pattern has no such group or the group did not participate in the
/// match.
fn group_text<'t>(caps: &StrCaptures<'t>, i: usize) -> &'t str {
    caps.get(i).map_or("", |m| m.as_str())
}
