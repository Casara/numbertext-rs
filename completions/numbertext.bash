# bash completion for numbertext.
#
# Install by sourcing it from ~/.bashrc, or dropping it into the
# bash-completion directory as `numbertext`:
#
#   . completions/numbertext.bash
#
# Language codes are not hard-coded here. They come from `--list` on the
# very binary being completed, so the suggestions match what that build
# actually embedded - a numbertext built with only `lang-en` completes
# only "en", with no second list to keep in step.

_numbertext() {
    local cur prev
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    case "$prev" in
        --lang)
            COMPREPLY=($(compgen -W "$("${COMP_WORDS[0]}" --list 2>/dev/null)" -- "$cur"))
            return
            ;;
        --cardinal | --ordinal | --ordinal-number | --year | --currency | --money | --convert)
            # A number, an ISO 4217 code, or a locale-defined section
            # name. None of the three is enumerable, so offer nothing
            # rather than something wrong.
            return
            ;;
    esac

    COMPREPLY=($(compgen -W "
        --lang
        --list
        --cardinal
        --ordinal
        --ordinal-number
        --year
        --currency
        --money
        --convert
        --help-section
        --help
    " -- "$cur"))
}

complete -F _numbertext numbertext
