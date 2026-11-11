//! Text output through runemark: verdict, counts, findings by outcome.

use a11y_report::{Outcome, Report};
use runemark::{
    ColorMode, Console, DetailLevel, Finding, FindingGroup, Metric, RenderOptions, Report as View,
    ScopeNote, Tone, Verdict,
};

pub fn print(target: &str, report: &Report) {
    let s = &report.summary;
    let verdict = if s.fail > 0 {
        Verdict::Failed
    } else if s.review > 0 {
        Verdict::Warning
    } else {
        Verdict::Passed
    };
    let mut view = View::new(format!("barrierlab {target}"), verdict)
        .with_detail_level(DetailLevel::Detailed)
        .add_metric(Metric::new("Fail", s.fail.to_string()))
        .add_metric(Metric::new("Review", s.review.to_string()))
        .add_metric(Metric::new("Untested", s.untested.to_string()));
    if s.rules_not_run > 0 {
        view = view.add_scope_note(ScopeNote::new(
            "Not run",
            vec![format!(
                "{} rules need data this tool does not have (rendered styles, geometry)",
                s.rules_not_run
            )],
        ));
    }
    if !report.inputs.is_empty() {
        let lines = report
            .inputs
            .iter()
            .map(|i| {
                let subject = i.subject.as_deref().unwrap_or("-");
                let detail = i.detail.as_deref().unwrap_or("");
                format!("{subject}: {detail}")
            })
            .collect();
        view = view.add_scope_note(ScopeNote::new(
            "Incomplete input (affected checks are untested, not passed)",
            lines,
        ));
    }
    for (outcome, title, tone, advisory) in [
        (Outcome::Fail, "Failed", Tone::Error, false),
        (Outcome::Review, "Needs review", Tone::Warning, false),
        (Outcome::Untested, "Check by hand", Tone::Muted, true),
    ] {
        let mut group = FindingGroup::new(title).with_advisory(advisory);
        for f in report.findings.iter().filter(|f| f.outcome == outcome) {
            group = group.add_finding(Finding::new(tone, &f.message).with_rule_id(&f.rule_id));
        }
        if !group.findings.is_empty() {
            view = view.add_group(group);
        }
    }
    // Width from COLUMNS (shells do not export it to child processes; set it
    // yourself for a fixed width, e.g. for snapshots).
    let width = std::env::var("COLUMNS")
        .ok()
        .and_then(|c| c.parse::<usize>().ok())
        .unwrap_or(0);
    let options = RenderOptions::new().with_width(width);
    print!(
        "{}",
        view.render_with_options(Console::stdout(ColorMode::Auto), options)
    );
}

/// The report as pretty JSON, every finding with its stable `id` first
/// (`Finding::id()`, which the model computes but does not store).
pub fn json(report: &Report) -> String {
    let mut value = serde_json::to_value(report).expect("serialize");
    if let Some(findings) = value["findings"].as_array_mut() {
        for (entry, finding) in findings.iter_mut().zip(&report.findings) {
            let serde_json::Value::Object(fields) = entry.take() else {
                unreachable!("a finding serialises as an object")
            };
            let mut with_id = serde_json::Map::new();
            with_id.insert("id".into(), finding.id().as_str().into());
            with_id.extend(fields);
            *entry = serde_json::Value::Object(with_id);
        }
    }
    serde_json::to_string_pretty(&value).expect("serialize")
}
