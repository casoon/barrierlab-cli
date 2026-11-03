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
