use std::io::{Read, Write};
use std::net::TcpListener;
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

/// Serves `tests/fixtures/site.css` at `/site.css` and 404 for everything
/// else, on an ephemeral local port. Returns the base URL.
fn serve() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut stream = stream;
            let mut buf = [0u8; 2048];
            let n = stream.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]);
            let (status, body) = if req.starts_with("GET /site.css ") {
                ("200 OK", include_str!("fixtures/site.css"))
            } else {
                ("404 Not Found", "not found")
            };
            let _ = write!(
                stream,
                "HTTP/1.1 {status}\r\ncontent-type: text/css\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    format!("http://127.0.0.1:{port}/")
}

fn json(args: &[&str]) -> (i32, serde_json::Value) {
    let (code, out) = run(args);
    (code, serde_json::from_str(&out).expect("json"))
}

fn rule_outcomes(v: &serde_json::Value, rule: &str) -> Vec<String> {
    v["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["rule_id"] == rule)
        .map(|f| f["outcome"].as_str().unwrap().to_string())
        .collect()
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
    let (_, v) = json(&["--json", "examples/page.html"]);
    assert!(v["summary"]["fail"].as_u64().unwrap() >= 5);
}

#[test]
fn missing_file_exits_two() {
    let (code, _) = run(&["no/such/file.html"]);
    assert_eq!(code, 2);
}

#[test]
fn linked_sheet_is_fetched_through_base_url() {
    let base = serve();
    let (code, v) = json(&["--json", "--base-url", &base, "tests/fixtures/linked.html"]);
    assert_eq!(code, 1);
    assert!(
        rule_outcomes(&v, "contrast/text-insufficient").contains(&"fail".to_string()),
        "{v}"
    );
    assert!(v["inputs"].as_array().is_none_or(|i| i.is_empty()), "{v}");
}

#[test]
fn local_file_without_base_url_notes_the_missing_sheet() {
    let (code, v) = json(&["--json", "tests/fixtures/linked.html"]);
    let notes = v["inputs"].as_array().expect("inputs");
    assert_eq!(notes.len(), 1, "{v}");
    assert_eq!(notes[0]["kind"], "stylesheet_unavailable");
    assert_eq!(notes[0]["subject"], "site.css");
    assert!(notes[0]["detail"].as_str().unwrap().contains("--base-url"));
    // Contrast is untested, not passed and not failed.
    assert!(!rule_outcomes(&v, "contrast/text-insufficient").contains(&"fail".to_string()));
    assert!(!rule_outcomes(&v, "contrast/text-insufficient").contains(&"pass".to_string()));
    assert_eq!(code, 0, "{v}");
}

#[test]
fn unreachable_sheet_is_a_note_with_the_reason() {
    let base = serve();
    let (code, v) = json(&["--json", "--base-url", &base, "tests/fixtures/missing.html"]);
    let notes = v["inputs"].as_array().expect("inputs");
    assert_eq!(notes.len(), 1, "{v}");
    assert_eq!(notes[0]["subject"], "missing.css");
    assert!(notes[0]["detail"].as_str().unwrap().contains("404"), "{v}");
    assert!(!rule_outcomes(&v, "contrast/text-insufficient").contains(&"fail".to_string()));
    assert_eq!(code, 0);
}

#[test]
fn fail_on_review_turns_review_findings_into_exit_one() {
    let page = "tests/fixtures/missing.html";
    let (_, v) = json(&["--json", page]);
    assert!(v["summary"]["review"].as_u64().unwrap() > 0, "{v}");
    assert_eq!(run(&[page]).0, 0);
    assert_eq!(run(&["--fail-on", "review", page]).0, 1);
}

#[test]
fn bad_flag_values_exit_two() {
    for args in [
        &["--fail-on", "never", "examples/page.html"][..],
        &["--width", "wide", "examples/page.html"],
        &["--base-url", "ftp://x/", "examples/page.html"],
        &["--width"],
    ] {
        assert_eq!(run(args).0, 2, "{args:?}");
    }
}

#[test]
fn dark_and_width_reach_the_environment() {
    // --dark and a narrow --width each switch on a media block with poor contrast.
    let dir = std::env::temp_dir().join(format!("barrierlab-env-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let page = dir.join("env.html");
    std::fs::write(
        &page,
        r#"<!doctype html><html lang="en"><head><title>t</title><style>
        p { color: #000; background: #fff; }
        @media (prefers-color-scheme: dark) { p { color: #777; background: #888; } }
        @media (max-width: 600px) { p { color: #aaa; background: #fff; } }
        </style></head><body><main><h1>t</h1><p>Some text here.</p></main></body></html>"#,
    )
    .unwrap();
    let page = page.to_str().unwrap();
    let light = json(&["--json", page]).1;
    let dark = json(&["--json", "--dark", page]).1;
    let narrow = json(&["--json", "--width", "500", page]).1;
    std::fs::remove_dir_all(&dir).ok();
    let rule = "contrast/text-insufficient";
    assert!(
        !rule_outcomes(&light, rule).contains(&"fail".to_string()),
        "{light}"
    );
    assert!(
        rule_outcomes(&dark, rule).contains(&"fail".to_string()),
        "{dark}"
    );
    assert!(
        rule_outcomes(&narrow, rule).contains(&"fail".to_string()),
        "{narrow}"
    );
}
