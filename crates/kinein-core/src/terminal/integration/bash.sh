# Kinein Vectis: bash integration, run as `--init-file`. Bash has already read
# the system bashrc (when built with one); the user's ~/.bashrc follows here,
# then a prompt marker (OSC 133;A) tells the IDE the shell is reading input.
if [ -r ~/.bashrc ]; then
    . ~/.bashrc
fi

# Last in PROMPT_COMMAND, after any theme rewrote PS1: the marker goes into
# PS1 itself, which readline prints once the terminal is in raw mode, so a
# command the IDE types then is not echoed twice.
__kinein_prompt_marker() {
    local status=$?
    case $PS1 in
        *'\e]133;A\a'*) ;;
        *) PS1='\[\e]133;A\a\]'$PS1 ;;
    esac
    return "$status"
}
if (( BASH_VERSINFO[0] > 5 || (BASH_VERSINFO[0] == 5 && BASH_VERSINFO[1] >= 1) )); then
    PROMPT_COMMAND+=(__kinein_prompt_marker)
else
    PROMPT_COMMAND="${PROMPT_COMMAND:+$PROMPT_COMMAND$'\n'}__kinein_prompt_marker"
fi
