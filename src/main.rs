//! `barrierlab <url|file>`: fetches a page, parses it without a browser and
//! checks it with the rules from `a11y-rules`.

mod fetch;
mod host;
mod output;

use a11y_report::Report;

const USAGE: &str = "usage: barrierlab [--json] [--bot] [--user-agent <ua>] <url|file>";

fn main() {
    let mut json = false;
    let mut opts = fetch::Options {
        bot: false,
        user_agent: None,
    };
    let mut target = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--bot" => opts.bot = true,
            "--user-agent" => opts.user_agent = args.next(),
            "-h" | "--help" => {
                println!("{USAGE}");
                return;
            }
            _ => target = Some(arg),
        }
    }
    let Some(target) = target else {
        eprintln!("{USAGE}");
        std::process::exit(2);
    };
    match run(&target, fetch::Fetcher::new(opts)) {
        Ok(report) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).expect("serialize")
                );
            } else {
                output::print(target.as_str(), &report);
            }
            std::process::exit(i32::from(report.summary.fail > 0));
        }
        Err(e) => {
            eprintln!("barrierlab: {e}");
            std::process::exit(2);
        }
    }
}

fn run(target: &str, fetcher: fetch::Fetcher) -> Result<Report, String> {
    let page = fetcher.page(target)?;
    let dom = host::arena(&page.html);
    let sheets = fetcher.stylesheets(&page, &host::style_sources(&dom));
    let host = host::Host::new(&dom, &sheets);
    let report = a11y_rules::run_full(&host);
    Ok(a11y_rules::run_stylesheets(report, &host, &sheets))
}
