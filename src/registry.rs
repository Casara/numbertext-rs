//! The process-wide locale registry: embedded sources, runtime
//! additions, and the compile-once cache in front of both.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use soros::Program;

use crate::error::Error;

// The `(code, source)` table for whichever locales this build embedded,
// written by build.rs from the selected `lang-*` features.
include!(concat!(env!("OUT_DIR"), "/embedded_locales.rs"));

/// Locale sources plus the programs compiled from them.
///
/// Both maps live under one lock so a source and its cached program can
/// never disagree - replacing a source has to invalidate its compiled
/// form in the same critical section.
#[derive(Default)]
struct Inner {
    sources: HashMap<String, Cow<'static, str>>,
    compiled: HashMap<String, Arc<Program>>,
}

/// The registry every entry point shares.
///
/// It exists once per process, which is exactly what
/// [`register_locale`](crate::register_locale) and
/// [`languages`](crate::languages) need: a locale registered anywhere is
/// visible everywhere, without threading a handle through every call.
static REGISTRY: LazyLock<RwLock<Inner>> = LazyLock::new(|| {
    let sources = EMBEDDED_LOCALES
        .iter()
        .map(|&(code, source)| (code.to_owned(), Cow::Borrowed(source)))
        .collect();

    RwLock::new(Inner {
        sources,
        compiled: HashMap::new(),
    })
});

/// Takes the read lock, recovering from poisoning.
///
/// A panic while the registry lock was held cannot have left the maps
/// inconsistent - every critical section here is a handful of infallible
/// map operations - so refusing to serve any locale for the rest of the
/// process would turn an unrelated panic elsewhere into a second,
/// avoidable failure.
fn read() -> RwLockReadGuard<'static, Inner> {
    REGISTRY
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Takes the write lock, recovering from poisoning. See [`read`].
fn write() -> RwLockWriteGuard<'static, Inner> {
    REGISTRY
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Returns the compiled program for `lang`, compiling and caching it on
/// first use.
///
/// A regional variant such as `"en-GB"` or `"pt-BR"` has no `.sor` file
/// of its own: it is a set of `[:lang-code:]`-tagged lines inside the
/// base language's file (spec 2.7.1) - `en.sor` carries en-AU/en-GB/
/// en-IE/en-NZ, `pt.sor` carries pt-BR. So when `lang` is not a known
/// source by itself, this falls back to the part before the first `-`
/// for the *source text*, while still compiling it with the full
/// requested code, which is what makes those tagged lines take effect.
///
/// # Errors
///
/// [`Error::UnknownLanguage`] if neither `lang` nor its base language
/// has a registered source.
pub(crate) fn program(lang: &str) -> Result<Arc<Program>, Error> {
    if let Some(program) = read().compiled.get(lang) {
        return Ok(Arc::clone(program));
    }

    let source = {
        let inner = read();
        resolve(lang)
            .into_iter()
            .find_map(|candidate| inner.sources.get(candidate.as_str()))
            .cloned()
    };

    let source = source.ok_or_else(|| Error::UnknownLanguage {
        lang: lang.to_owned(),
    })?;

    // Deliberately compiled outside the lock: compiling a locale is by
    // far the most expensive thing this module does, and holding the
    // write lock across it would serialize every unrelated lookup behind
    // it. Two threads racing on the same new locale may both compile it
    // and one result is dropped - cheaper than the contention, and the
    // programs are equivalent either way.
    let program = Arc::new(Program::compile(&source, lang));

    write()
        .compiled
        .insert(lang.to_owned(), Arc::clone(&program));

    Ok(program)
}

/// The source keys to try for `lang`, in order, mirroring upstream
/// libnumbertext's `Numbertext::load` (`src/Numbertext.cxx`).
///
/// Only the *source* is resolved this way. Whatever is found is always
/// compiled with the full requested code, which is what lets a base
/// file's `[:lang-code:]`-tagged lines take effect (spec 2.7.1).
///
/// 1. The code exactly as given. Upstream has no such step - it looks up
///    files - but this registry also holds codes added at runtime by
///    [`crate::register_locale`], which may contain a `-` that must not
///    be rewritten out from under the caller.
/// 2. The code with `-` rewritten to `_`, because upstream names the
///    script-variant files that way: `hu-Hung` has to reach
///    `hu_Hung.sor`, not `hu.sor`. Without this step the request
///    silently returns plain Hungarian instead of the Old Hungarian
///    script - wrong output rather than an error, which is worse.
/// 3. The base language, before the first `-`, which is how the ordinary
///    regional variants resolve: `en-GB` and `pt-BR` read `en` and `pt`.
/// 4. Norwegian, which upstream special-cases by name: Bokmål and
///    Nynorsk both live in `no.sor`, and neither `nb` nor `nn` has a file
///    of its own. Without this, `nn` is simply rejected and the
///    `[:nn:]`/`[:nn-NO:]` lines in `no.sor` are unreachable by any
///    code at all.
fn resolve(lang: &str) -> Vec<String> {
    let mut candidates = vec![lang.to_owned()];

    if lang.contains('-') {
        candidates.push(lang.replace('-', "_"));
    }

    if let Some((base, _)) = lang.split_once('-') {
        candidates.push(base.to_owned());
    }

    if is_norwegian(lang) {
        candidates.push("no".to_owned());
    }

    candidates
}

/// Upstream matches Norwegian with the regular expression
/// `n[bn]([-_]NO)?` (`LANG_PATTERN_NO`). Enumerating the six strings it
/// accepts keeps this crate free of a regex dependency of its own -
/// `numbertext` depends on nothing but `soros`.
fn is_norwegian(lang: &str) -> bool {
    matches!(lang, "nb" | "nn" | "nb-NO" | "nn-NO" | "nb_NO" | "nn_NO")
}

/// Adds or replaces a locale's source, invalidating any program already
/// compiled from the previous one.
pub(crate) fn register(code: &str, source: String) {
    let mut inner = write();
    inner.sources.insert(code.to_owned(), Cow::Owned(source));
    inner.compiled.remove(code);
}

/// Every known language code, sorted.
pub(crate) fn languages() -> Vec<String> {
    let mut codes: Vec<String> = read().sources.keys().cloned().collect();
    codes.sort();

    codes
}

/// Every variant code the known sources declare through
/// `[:lang-code:]`-tagged lines, sorted, excluding any that is already a
/// source in its own right.
///
/// These are read out of the rule files rather than listed here, so a
/// data sync that adds or drops a variant is picked up without a code
/// change - the same property that makes adding a language a data
/// change.
pub(crate) fn language_variants() -> Vec<String> {
    let inner = read();
    let mut variants: Vec<String> = inner
        .sources
        .values()
        .flat_map(|source| soros::language_tags(source))
        .filter(|tag| !inner.sources.contains_key(tag))
        .collect();

    variants.sort();
    variants.dedup();

    variants
}
