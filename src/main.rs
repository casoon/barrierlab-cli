//! `barrierlab <url|file>`: fetches a page and its stylesheets and hands them
//! to `a11y-static`, which parses and checks them without a browser.

mod fetch;
mod output;

use a11y_report::Report;
use a11y_static::{Environment, Page};
use url::Url;

const USAGE: &str = "usage: barrierlab [--json] [--bot] [--user-agent <ua>] [--base-url <url>]
                  [--width <px>] [--dark] [--fail-on <fail|review>] <url|file>
       barrierlab --version";

#[derive(Clone, Copy, PartialEq)]
enum FailOn {
    Fail,
    Review,
}

struct Args {
    json: bool,
    opts: fetch::Options,
    base: Option<Url>,
    env: Environment,
    fail_on: FailOn,
    target: String,
}

fn value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next().ok_or_else(|| format!("{flag} needs a value"))
}

fn parse_args() -> Result<Option<Args>, String> {
    let mut a = Args {
        json: false,
        opts: fetch::Options {
            bot: false,
            user_agent: None,
        },
        base: None,
        env: Environment::default(),
        fail_on: FailOn::Fail,
        target: String::new(),
    };
    let mut target = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => a.json = true,
            "--bot" => a.opts.bot = true,
            "--dark" => a.env.dark = true,
            "--user-agent" => a.opts.user_agent = Some(value(&mut args, "--user-agent")?),
            "--base-url" => {
                let v = value(&mut args, "--base-url")?;
                let url = Url::parse(&v).map_err(|e| format!("--base-url {v}: {e}"))?;
                if !matches!(url.scheme(), "http" | "https") {
                    return Err(format!("--base-url {v}: only http and https"));
                }
                a.base = Some(url);
            }
            "--width" => {
                let v = value(&mut args, "--width")?;
                a.env.width = v
                    .parse::<f32>()
                    .ok()
                    .filter(|w| w.is_finite() && *w > 0.0)
                    .ok_or_else(|| format!("--width {v}: not a positive number"))?;
            }
            "--fail-on" => {
                a.fail_on = match value(&mut args, "--fail-on")?.as_str() {
                    "fail" => FailOn::Fail,
                    "review" => FailOn::Review,
                    v => return Err(format!("--fail-on {v}: expected fail or review")),
                };
            }
            "-V" | "--version" => {
                println!("barrierlab {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(None);
            }
            _ => target = Some(arg),
        }
    }
    a.target = target.ok_or_else(|| USAGE.to_string())?;
    Ok(Some(a))
}

fn main() {
    let args = match parse_args() {
        Ok(Some(a)) => a,
        Ok(None) => return,
        Err(e) => {
            eprintln!("barrierlab: {e}");
            std::process::exit(2);
        }
    };
    match run(&args) {
        Ok(report) => {
            if args.json {
                println!("{}", output::json(&report));
            } else {
                output::print(&args.target, &report);
            }
            let s = &report.summary;
            let failed = s.fail > 0 || (args.fail_on == FailOn::Review && s.review > 0);
            std::process::exit(i32::from(failed));
        }
        Err(e) => {
            eprintln!("barrierlab: {e}");
            std::process::exit(2);
        }
    }
}

fn run(args: &Args) -> Result<Report, String> {
    let fetcher = fetch::Fetcher::new(fetch::Options {
        bot: args.opts.bot,
        user_agent: args.opts.user_agent.clone(),
    });
    let page = fetcher.page(&args.target, args.base.clone())?;
    let doc = Page::parse(&page.html);
    let sheets = fetcher.stylesheets(&page, &doc.stylesheet_refs());
    let report = doc.audit(&sheets, args.env, a11y_static::Locale::En);
    Ok(report)
}
