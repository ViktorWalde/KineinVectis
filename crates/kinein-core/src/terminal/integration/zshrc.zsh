# Kinein Vectis: zsh integration, second file (see .zshenv).
if (( ${+KINEIN_ZDOTDIR} )); then
    ZDOTDIR=$KINEIN_ZDOTDIR
else
    unset ZDOTDIR
fi
unset KINEIN_ZDOTDIR

if [[ -r ${ZDOTDIR:-$HOME}/.zshrc ]]; then
    source ${ZDOTDIR:-$HOME}/.zshrc
fi

# The marker (OSC 133;A) goes out when the line editor starts reading the
# main prompt: after every precmd hook, a theme's wizard included.
_kinein_prompt_marker() {
    builtin print -n '\e]133;A\a'
}
_kinein_line_init() {
    if [[ $CONTEXT == start ]]; then
        _kinein_prompt_marker
    fi
}
# A .zshrc that never binds a key has not loaded the line editor yet, and
# add-zle-hook-widget gives up without it.
if [[ -o zle ]] && zmodload zsh/zle 2>/dev/null \
    && autoload -Uz +X add-zle-hook-widget 2>/dev/null; then
    add-zle-hook-widget line-init _kinein_line_init
else
    precmd_functions+=(_kinein_prompt_marker)
fi
