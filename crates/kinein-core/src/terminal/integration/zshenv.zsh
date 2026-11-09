# Kinein Vectis: zsh integration. ZDOTDIR points here only so zsh reads these
# two files; each one gives the user's ZDOTDIR back before reading theirs.
if (( ${+KINEIN_ZDOTDIR} )); then
    ZDOTDIR=$KINEIN_ZDOTDIR
else
    unset ZDOTDIR
fi
unset KINEIN_ZDOTDIR

# zsh offers zsh-newuser-install before reading .zshenv, when the startup
# directory has no startup file; it looked here. The same module asks again
# for the user's directory.
if [[ -o interactive ]]; then
    zmodload zsh/newuser 2>/dev/null && zmodload -u zsh/newuser 2>/dev/null
fi

if [[ -r ${ZDOTDIR:-$HOME}/.zshenv ]]; then
    source ${ZDOTDIR:-$HOME}/.zshenv
fi

if [[ -o interactive ]]; then
    (( ${+ZDOTDIR} )) && KINEIN_ZDOTDIR=$ZDOTDIR
    ZDOTDIR=${${(%):-%x}:A:h}
fi
