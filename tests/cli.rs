//! End-to-end CLI tests using the compiled binary.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

/// Run `papercut` against a throwaway journal in `dir`, with fixed clock/user.
fn pc(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("papercut").unwrap();
    cmd.current_dir(dir)
        .env("PAPERCUTS_FILE", dir.join("log.jsonl"))
        .env("PAPERCUTS_NOW", "2026-01-01T00:00:00.000Z")
        .env("PAPERCUTS_USER", "tester@example.com")
        .env("PAPERCUTS_HARNESS", "test-harness")
        .env("PAPERCUTS_MODEL", "test-model");
    cmd
}

fn json_of(stdout: &str) -> Value {
    serde_json::from_str(stdout.trim()).unwrap()
}

#[test]
fn add_writes_record_and_returns_envelope() {
    let dir = tempdir().unwrap();
    let out = pc(dir.path())
        .args(["add", "yarn web:test finds no files", "--tag", "tooling"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v = json_of(&String::from_utf8(out.stdout).unwrap());
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["changed"], true);
    assert_eq!(v["data"]["record"]["kind"], "cut");
    assert_eq!(v["data"]["record"]["text"], "yarn web:test finds no files");
    assert_eq!(v["data"]["record"]["user"], "tester@example.com");
    assert_eq!(v["data"]["record"]["model"], "test-model");
    assert_eq!(v["data"]["record"]["harness"], "test-harness");
    assert_eq!(v["data"]["record"]["tags"][0], "tooling");
    assert!(v["data"]["record"]["id"]
        .as_str()
        .unwrap()
        .starts_with("pc_"));

    // File exists and contains exactly one JSON line.
    let file = fs::read_to_string(dir.path().join("log.jsonl")).unwrap();
    assert_eq!(file.trim().lines().count(), 1);
}

#[test]
fn add_is_duplicate_safe() {
    let dir = tempdir().unwrap();
    let mut cmd_a = pc(dir.path());
    let mut cmd_b = pc(dir.path());
    cmd_a.args(["add", "same complaint", "-t", "x"]);
    cmd_b.args(["add", "same complaint", "-t", "x"]);
    let o1 = cmd_a.output().unwrap();
    let o2 = cmd_b.output().unwrap();
    assert!(o1.status.success() && o2.status.success());
    let v1 = json_of(&String::from_utf8(o1.stdout).unwrap());
    let v2 = json_of(&String::from_utf8(o2.stdout).unwrap());
    assert_eq!(v1["data"]["changed"], true);
    assert_eq!(v2["data"]["changed"], false, "second identical add is a no-op");
    assert_eq!(v1["data"]["record"]["id"], v2["data"]["record"]["id"]);
    let file = fs::read_to_string(dir.path().join("log.jsonl")).unwrap();
    assert_eq!(file.trim().lines().count(), 1, "only one line written");
}

#[test]
fn add_reads_stdin() {
    let dir = tempdir().unwrap();
    let mut cmd = pc(dir.path());
    cmd.arg("add").arg("-").arg("-t").arg("stdin");
    cmd.write_stdin("a problem surfaced only via stdin");
    let out = cmd.output().unwrap();
    assert!(out.status.success());
    let v = json_of(&String::from_utf8(out.stdout).unwrap());
    assert_eq!(
        v["data"]["record"]["text"],
        "a problem surfaced only via stdin"
    );
}

#[test]
fn add_rejects_empty_body() {
    let dir = tempdir().unwrap();
    let out = pc(dir.path()).arg("add").arg("   ").output().unwrap();
    assert_eq!(out.status.code(), Some(65)); // EX_BAD_INPUT
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("E_BAD_INPUT"), "{stderr}");
}

#[test]
fn add_is_implicit_default_subcommand() {
    let dir = tempdir().unwrap();
    // `papercut "msg"` — no subcommand; message is the first positional.
    let out = pc(dir.path()).arg("bare message").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v = json_of(&String::from_utf8(out.stdout).unwrap());
    assert_eq!(v["data"]["record"]["text"], "bare message");
}

#[test]
fn short_model_flag_overrides_detection() {
    let dir = tempdir().unwrap();
    // `papercut -m gpt-5 "msg"` — implicit add with a model override.
    let out = pc(dir.path())
        .args(["-m", "gpt-5", "footgun via -m"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v = json_of(&String::from_utf8(out.stdout).unwrap());
    assert_eq!(v["data"]["record"]["model"], "gpt-5");
}

#[test]
fn bare_invocation_prints_help() {
    let dir = tempdir().unwrap();
    let out = pc(dir.path()).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Usage: papercut"), "{stdout}");
}

#[test]
fn list_returns_open_cuts_json() {
    let dir = tempdir().unwrap();
    pc(dir.path())
        .args(["add", "first"])
        .output()
        .unwrap();
    pc(dir.path())
        .args(["add", "second", "-t", "docs"])
        .output()
        .unwrap();
    let out = pc(dir.path())
        .args(["list", "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v = json_of(&String::from_utf8(out.stdout).unwrap());
    assert_eq!(v["data"]["count"], 2);
    assert_eq!(v["data"]["records"].as_array().unwrap().len(), 2);
}

#[test]
fn list_default_is_human_readable_text() {
    let dir = tempdir().unwrap();
    pc(dir.path())
        .args(["add", "a doc footgun", "-t", "docs"])
        .output()
        .unwrap();
    let out = pc(dir.path()).arg("list").output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        !text.trim_start().starts_with('{'),
        "default list should be human-readable, not JSON: {text}"
    );
    assert!(text.contains("a doc footgun"));
    assert!(text.contains("open papercut"));
}

#[test]
fn list_markdown_format() {
    let dir = tempdir().unwrap();
    pc(dir.path()).args(["add", "a doc footgun", "-t", "docs"]).output().unwrap();
    let out = pc(dir.path())
        .args(["list", "--format", "md"])
        .output()
        .unwrap();
    let md = String::from_utf8(out.stdout).unwrap();
    assert!(md.contains("a doc footgun"));
    assert!(md.contains("## Open"));
}

#[test]
fn resolve_marks_fixed_and_is_idempotent() {
    let dir = tempdir().unwrap();
    let added = pc(dir.path()).args(["add", "resolve me"]).output().unwrap();
    let id = json_of(&String::from_utf8(added.stdout).unwrap())["data"]["record"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let r1 = pc(dir.path()).args(["resolve", &id]).output().unwrap();
    assert!(r1.status.success());
    let v1 = json_of(&String::from_utf8(r1.stdout).unwrap());
    assert_eq!(v1["data"]["changed"], true);
    assert_eq!(v1["data"]["id"], id);

    let r2 = pc(dir.path()).args(["resolve", &id]).output().unwrap();
    let v2 = json_of(&String::from_utf8(r2.stdout).unwrap());
    assert_eq!(v2["data"]["changed"], false, "resolving twice is a no-op");

    // It disappears from default list, but appears with --all.
    let list = pc(dir.path())
        .args(["list", "--format", "json"])
        .output()
        .unwrap();
    let lv = json_of(&String::from_utf8(list.stdout).unwrap());
    assert_eq!(lv["data"]["count"], 0);
    let all = pc(dir.path())
        .args(["list", "--all", "--format", "json"])
        .output()
        .unwrap();
    let av = json_of(&String::from_utf8(all.stdout).unwrap());
    assert_eq!(av["data"]["count"], 1);
}

#[test]
fn resolve_supports_unique_prefix() {
    let dir = tempdir().unwrap();
    let added = pc(dir.path()).args(["add", "prefix test"]).output().unwrap();
    let v = json_of(&String::from_utf8(added.stdout).unwrap());
    let id = v["data"]["record"]["id"].as_str().unwrap().to_string();
    let prefix = id[..7].to_string();
    let out = pc(dir.path()).args(["resolve", &prefix]).output().unwrap();
    assert!(out.status.success());
    let v = json_of(&String::from_utf8(out.stdout).unwrap());
    assert_eq!(v["data"]["changed"], true);
}

#[test]
fn resolve_unknown_id_exits_66() {
    let dir = tempdir().unwrap();
    let out = pc(dir.path()).args(["resolve", "pc_doesnotexist"]).output().unwrap();
    assert_eq!(out.status.code(), Some(66)); // EX_NOT_FOUND
    assert!(String::from_utf8(out.stderr).unwrap().contains("E_NOT_FOUND"));
}

#[test]
fn schema_returns_contract() {
    let dir = tempdir().unwrap();
    let out = pc(dir.path()).arg("schema").output().unwrap();
    assert!(out.status.success());
    let v = json_of(&String::from_utf8(out.stdout).unwrap());
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["contract"], 1);
    assert!(v["data"]["commands"]["add"]["write"] == true);
}

#[test]
fn doctor_reports_ok_on_clean_journal() {
    let dir = tempdir().unwrap();
    pc(dir.path()).args(["add", "hello"]).output().unwrap();
    let out = pc(dir.path()).arg("doctor").output().unwrap();
    assert!(out.status.success());
    let v = json_of(&String::from_utf8(out.stdout).unwrap());
    assert_eq!(v["data"]["ok"], true);
    assert_eq!(v["data"]["records"], 1);
    assert_eq!(v["data"]["open"], 1);
    assert_eq!(v["data"]["torn"], 0);
}
