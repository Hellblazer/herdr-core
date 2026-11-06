#![cfg(unix)] // The hook finds Claude's terminal through ps and /dev; Windows has neither.

mod common;

use common::*;
use serde_json::Value;

const HOOK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../integrations/claude-code/hc-status.sh");

fn wait_record(s: &Session, t: &str, what: &str, ok: impl Fn(&Value) -> bool) -> Value {
    let mut last = Value::Null;
    wait_until(what, || {
        last = get(s, t);
        ok(&last)
    });
    last
}

/// The hook, started the way Claude Code starts one: in its own session, so
/// it has no controlling terminal, with the event's JSON on stdin.
/// `h <name> <args>` runs it once file <name> exists; `n` runs it without
/// HC_TERMINAL_ID, as outside hc.
fn hooked(s: &Session, body: &str) -> String {
    let detached = "import os, sys\nos.setsid()\nos.execvp(sys.argv[1], sys.argv[1:])";
    let h = format!(
        "h(){{ w $1; shift; echo '{{}}' | python3 -c '{detached}' sh {HOOK} \"$@\" >/dev/null 2>&1; }}; \
         n(){{ w $1; shift; echo '{{}}' | env -u HC_TERMINAL_ID python3 -c '{detached}' sh {HOOK} \"$@\" >/dev/null 2>&1; }}; \
         python3 -c '{detached}' sh -c ': </dev/tty' 2>/dev/null && touch {}/has-tty; ",
        s.root.display()
    );
    script(s, &format!("{h}{body}"))
}

#[test]
fn hook_reports_claude_status_through_its_terminal_without_a_tty_of_its_own() {
    let s = Session::new();
    let body = "h a idle; h b working; h c blocked permission; h d done; n e working; h f idle; h g clear; sleep 100";
    let t = run(&s, &["sh", "-c", &hooked(&s, body)]);

    touch(&s, "a");
    let rec = wait_record(&s, &t, "idle", |r| r["status"]["state"] == "idle");
    assert!(!s.root.join("has-tty").exists(), "the hook must run with no controlling terminal, as under Claude");
    assert_eq!(rec["status"]["app"], "claude-code");

    touch(&s, "b");
    wait_record(&s, &t, "working", |r| r["status"]["state"] == "working");
    touch(&s, "c");
    let rec = wait_record(&s, &t, "blocked", |r| r["status"]["state"] == "blocked");
    assert_eq!(rec["status"]["kind"], "permission");
    touch(&s, "d");
    let rec = wait_record(&s, &t, "done", |r| r["status"]["state"] == "done");
    let turn = rec["turn"].as_u64();

    // Outside hc it reports nothing: a "working" there would end another turn at the idle.
    touch(&s, "e");
    touch(&s, "f");
    let rec = wait_record(&s, &t, "idle again", |r| r["status"]["state"] == "idle");
    assert_eq!(rec["turn"].as_u64(), turn, "the hook without HC_TERMINAL_ID wrote a report");

    touch(&s, "g");
    wait_record(&s, &t, "cleared", |r| r["status"].is_null());
    assert!(!text(&s, &t).contains("7501"), "reports never reach the screen");
}
