# Claude Code status for herdr-core

Claude Code doesn't send OSC 7501, so under `hc` it has no status. These
hooks report it: `working` while Claude works, `blocked` at a permission
prompt or question, `done` when a turn ends, `error` when one fails on an API
error. `hc wait`, `hc get` and `hc events` then work for Claude like for any
program that reports.

`hc-status.sh` does nothing outside an hc terminal (no `HC_TERMINAL_ID`).
Claude Code runs hooks with no controlling terminal, so the script writes the
report to the terminal of its nearest ancestor that has one, Claude's own.
macOS and Linux only; it needs `ps`.

## Install

Copy `hc-status.sh` somewhere stable, make it executable, and add the hooks
below to `~/.claude/settings.json`, with `HC_STATUS` replaced by the script's
absolute path. Merge them into any `hooks` you already have.

```json
{
  "hooks": {
    "SessionStart": [{ "hooks": [{ "type": "command", "command": "HC_STATUS idle" }] }],
    "UserPromptSubmit": [{ "hooks": [{ "type": "command", "command": "HC_STATUS working" }] }],
    "PreToolUse": [{ "hooks": [{ "type": "command", "command": "HC_STATUS working" }] }],
    "PostToolUse": [{ "hooks": [{ "type": "command", "command": "HC_STATUS working" }] }],
    "PostToolUseFailure": [{ "hooks": [{ "type": "command", "command": "HC_STATUS working" }] }],
    "PermissionRequest": [{ "hooks": [{ "type": "command", "command": "HC_STATUS blocked permission" }] }],
    "Notification": [
      { "matcher": "permission_prompt", "hooks": [{ "type": "command", "command": "HC_STATUS blocked permission" }] },
      { "matcher": "elicitation_dialog", "hooks": [{ "type": "command", "command": "HC_STATUS blocked question" }] },
      { "matcher": "elicitation_complete|elicitation_response", "hooks": [{ "type": "command", "command": "HC_STATUS working" }] }
    ],
    "Stop": [{ "hooks": [{ "type": "command", "command": "HC_STATUS done" }] }],
    "StopFailure": [{ "hooks": [{ "type": "command", "command": "HC_STATUS error" }] }],
    "SessionEnd": [{ "hooks": [{ "type": "command", "command": "HC_STATUS clear" }] }]
  }
}
```

A report written while Claude repaints can land inside one of its frames and
garble that frame until the next repaint.
