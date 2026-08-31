# fish completion for numbertext.
#
# Install by copying it into fish's completion directory:
#
#   cp completions/numbertext.fish ~/.config/fish/completions/
#
# Language codes come from `--list` on the binary being completed, so the
# suggestions match what that build actually embedded.

# No file completion: every argument is a number, a code, or a section
# name, never a path.
complete -c numbertext -f

complete -c numbertext -l lang -r -d 'Language code, default "en"' \
    -a "(numbertext --list 2>/dev/null)"
complete -c numbertext -l list           -d 'List available language codes'
complete -c numbertext -l cardinal       -r -d 'Spell out a cardinal number'
complete -c numbertext -l ordinal        -r -d 'Spell out an ordinal number'
complete -c numbertext -l ordinal-number -r -d 'Format an abbreviated ordinal'
complete -c numbertext -l year           -r -d 'Spell out a calendar year'
complete -c numbertext -l currency       -r -d 'Currency amount: <ISO> <n>'
complete -c numbertext -l money          -r -d 'MONEYTEXT-style amount: <ISO> <n>'
complete -c numbertext -l convert        -r -d 'Locale-specific section: <name> <n>'
complete -c numbertext -l help-section   -d "Print the locale's own usage summary"
complete -c numbertext -l help           -d 'Print usage'
