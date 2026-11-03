use std::process::Command;

fn run(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_barrierlab"))
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("run barrierlab");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn reports_defects_and_exits_one() {
    let (code, text) = run(&["examples/page.html"]);
    assert_eq!(code, 1);
    assert!(text.contains("images/alt-missing"), "{text}");
    assert!(text.contains("links/name-missing"), "{text}");
    assert!(text.contains("contrast/text-insufficient"), "{text}");
}

#[test]
fn json_output_carries_the_report() {
    let (_, json) = run(&["--json", "examples/page.html"]);
    let v: serde_json::Value = serde_json::from_str(&json).expect("json");
    assert!(v["summary"]["fail"].as_u64().unwrap() >= 5);
}

#[test]
fn missing_file_exits_two() {
    let (code, _) = run(&["no/such/file.html"]);
    assert_eq!(code, 2);
}
