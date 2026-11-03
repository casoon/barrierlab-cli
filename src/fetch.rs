//! Abruf der Seite und ihrer Stylesheets. Beschafft nur; bewertet nicht.
//!
//! Anfragen sehen aus wie die eines normalen Browsers (Accept, Sprache,
//! Kompression, Navigations-Header), weil viele Server nackte Clients
//! abweisen. Eine Schutzseite oder Sperre wird gemeldet, nicht umgangen.

use std::thread::sleep;
use std::time::Duration;

use stylesheet_parse::{Stylesheet, parse_stylesheet};
use ureq::Agent;
use url::Url;

const BROWSER_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
const BOT_UA: &str = concat!(
    "barrierlab/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/casoon/barrierlab-cli)"
);
const TRIES: u32 = 3;
const BACKOFF_MS: u64 = 400;
/// Längste Wartezeit, die ein `Retry-After` uns abverlangen darf.
const RETRY_AFTER_MAX_S: u64 = 10;

pub struct Options {
    /// Sich ehrlich als `barrierlab` ausweisen statt als Browser.
    pub bot: bool,
    pub user_agent: Option<String>,
}

pub struct Fetcher {
    agent: Agent,
    bot: bool,
}

pub struct Page {
    pub html: String,
    /// Basis zum Auflösen von `<link href>`; bei einer Datei `None`.
    pub base: Option<Url>,
}

#[derive(Clone, Copy)]
enum Kind {
    Document,
    Style,
}

impl Fetcher {
    pub fn new(opts: Options) -> Self {
        let ua = opts
            .user_agent
            .unwrap_or_else(|| if opts.bot { BOT_UA } else { BROWSER_UA }.to_string());
        let agent: Agent = Agent::config_builder()
            .user_agent(ua)
            .timeout_global(Some(Duration::from_secs(30)))
            .max_redirects(10)
            // Status selbst auswerten, um Wiederholung und Hinweise zu steuern.
            .http_status_as_error(false)
            .build()
            .into();
        Fetcher {
            agent,
            bot: opts.bot,
        }
    }

    pub fn page(&self, target: &str) -> Result<Page, String> {
        if let Ok(url) = Url::parse(target)
            && matches!(url.scheme(), "http" | "https")
        {
            let html = self.get(&url, Kind::Document, None)?;
            return Ok(Page {
                html,
                base: Some(url),
            });
        }
        let html = std::fs::read_to_string(target).map_err(|e| format!("{target}: {e}"))?;
        Ok(Page { html, base: None })
    }

    /// Die Stylesheets der Seite in Dokumentreihenfolge: `<style>`-Text und
    /// verlinkte Dateien. Eine nicht ladbare Datei fehlt, mit Hinweis auf stderr.
    pub fn stylesheets(&self, page: &Page, inline: &[Inline]) -> Vec<Stylesheet> {
        inline
            .iter()
            .filter_map(|s| match s {
                Inline::Style(css) => Some(parse_stylesheet(css)),
                Inline::Link(href) => {
                    let base = page.base.as_ref()?;
                    let url = base.join(href).ok()?;
                    match self.get(&url, Kind::Style, Some(base)) {
                        Ok(css) => Some(parse_stylesheet(&css)),
                        Err(e) => {
                            eprintln!("barrierlab: stylesheet skipped: {e}");
                            None
                        }
                    }
                }
            })
            .collect()
    }

    /// Ein GET mit Wiederholung bei Übertragungsfehlern, 429 und 5xx.
    fn get(&self, url: &Url, kind: Kind, referer: Option<&Url>) -> Result<String, String> {
        let mut last = String::new();
        for attempt in 0..TRIES {
            let mut req = self.agent.get(url.as_str());
            for (k, v) in self.headers(kind, referer) {
                req = req.header(k, v);
            }
            match req.call() {
                Ok(mut resp) => {
                    let status = resp.status().as_u16();
                    if (200..300).contains(&status) {
                        return resp
                            .body_mut()
                            .read_to_string()
                            .map_err(|e| format!("{url}: {e}"));
                    }
                    last = status_message(url, status);
                    if !retryable(status) {
                        return Err(last);
                    }
                    let wait = retry_after(&resp)
                        .unwrap_or_else(|| Duration::from_millis(BACKOFF_MS << attempt));
                    if attempt + 1 < TRIES {
                        sleep(wait);
                    }
                }
                Err(e) => {
                    last = format!("{url}: {e}");
                    if attempt + 1 < TRIES {
                        sleep(Duration::from_millis(BACKOFF_MS << attempt));
                    }
                }
            }
        }
        Err(last)
    }

    fn headers(&self, kind: Kind, referer: Option<&Url>) -> Vec<(&'static str, String)> {
        let accept = match kind {
            Kind::Document => "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            Kind::Style => "text/css,*/*;q=0.1",
        };
        let mut h = vec![
            ("accept", accept.to_string()),
            ("accept-language", "en-US,en;q=0.9,de;q=0.8".to_string()),
        ];
        if self.bot {
            return h;
        }
        let (dest, site) = match kind {
            Kind::Document => ("document", "none"),
            Kind::Style => ("style", "same-origin"),
        };
        h.push(("sec-fetch-dest", dest.to_string()));
        h.push(("sec-fetch-mode", "navigate".to_string()));
        h.push(("sec-fetch-site", site.to_string()));
        if matches!(kind, Kind::Document) {
            h.push(("sec-fetch-user", "?1".to_string()));
            h.push(("upgrade-insecure-requests", "1".to_string()));
        }
        if let Some(r) = referer {
            h.push(("referer", r.to_string()));
        }
        h
    }
}

fn retryable(status: u16) -> bool {
    matches!(status, 429 | 500 | 502 | 503 | 504)
}

fn retry_after(resp: &ureq::http::Response<ureq::Body>) -> Option<Duration> {
    let s: u64 = resp
        .headers()
        .get("retry-after")?
        .to_str()
        .ok()?
        .trim()
        .parse()
        .ok()?;
    Some(Duration::from_secs(s.min(RETRY_AFTER_MAX_S)))
}

fn status_message(url: &Url, status: u16) -> String {
    let hint = match status {
        401 | 403 | 429 => {
            ": the server refuses this client (login or bot protection). \
             Save the page from a browser and pass the file instead"
        }
        404 | 410 => ": page not found",
        _ => "",
    };
    format!("{url}: HTTP {status}{hint}")
}

pub enum Inline {
    Style(String),
    Link(String),
}
