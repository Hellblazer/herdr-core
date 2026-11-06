#!/bin/sh
# Claude Code hook: tells herdr-core what Claude is doing, as OSC 7501.
#   hc-status.sh <idle|working|done|blocked|error|clear> [permission|question|auth]
# Claude Code (2.1.139+) runs hooks with no controlling terminal, and its
# terminalSequence output does not pass 7501, so the report goes to the
# terminal of the nearest ancestor that has one: Claude's. Prints nothing:
# some hooks' stdout becomes Claude's context.

[ -t 0 ] || cat >/dev/null 2>&1 # the event's JSON; the state comes from the arguments
[ -n "${HC_TERMINAL_ID:-}" ] || exit 0
state=${1:-} kind=${2:-}
case $state in idle | working | done | blocked | error | clear) ;; *) exit 0 ;; esac
case $kind in '' | permission | question | auth) ;; *) exit 0 ;; esac

pid=$PPID depth=0 tty=
while [ -z "$tty" ] && [ "$depth" -lt 16 ] && [ "${pid:-0}" -gt 1 ]; do
    set -- $(ps -o tty= -o ppid= -p "$pid" 2>/dev/null)
    case ${1:-?} in
    '?' | '??' | '-') pid=${2:-0} ;;
    *) tty=/dev/$1 ;;
    esac
    depth=$((depth + 1))
done
[ -n "$tty" ] && [ -w "$tty" ] || exit 0

printf '\033]7501;state=%s:app=claude-code%s\033\\' "$state" "${kind:+:kind=$kind}" >"$tty" 2>/dev/null
exit 0
