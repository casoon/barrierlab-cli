//! Fetching the page and its stylesheets. Fetches only; judges nothing.
//!
//! Requests look like those of an ordinary browser (Accept, language,
//! compression, navigation headers) because many servers turn away bare
//! clients. A protection page or a refusal is reported, not worked around.

use std::thread::sleep;
use std::time::Duration;

use a11y_static::{Sheet, StylesheetRef};
use ureq::Agent;
use url::Url;

const BROWSER_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
const BOT_UA: &str = concat!(
    "barrierlab/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/casoon/barrierlab-cli)"
);
/// Upper bounds for a response body; larger ones are an error, not truncated.
const MAX_PAGE_BYTES: u64 = 10 * 1024 * 1024;
const MAX_SHEET_BYTES: u64 = 2 * 1024 * 1024;
const TRIES: u32 = 3;
const BACKOFF_MS: u64 = 400;
/// The longest wait a `Retry-After` may ask of us.
const RETRY_AFTER_MAX_S: u64 = 10;

pub struct Options {
    /// Identify honestly as `barrierlab` instead of as a browser.
    pub bot: bool,
    pub user_agent: Option<String>,
}

pub struct Fetcher {
    agent: Agent,
    bot: bool,
}

pub struct Page {
    pub html: String,
    /// Base for resolving `<link href>`; `None` for a file without `--base-url`.
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
            // Evaluate the status ourselves to drive retries and hints.
            .http_status_as_error(false)
            .build()
            .into();
        Fetcher {
            agent,
            bot: opts.bot,
        }
    }

    /// Loads the target. `base` overrides the base for resolving `<link href>`;
    /// it is the only way to give a local file one.
    pub fn page(&self, target: &str, base: Option<Url>) -> Result<Page, String> {
        if let Ok(url) = Url::parse(target)
            && matches!(url.scheme(), "http" | "https")
        {
            let html = self.get(&url, Kind::Document, None)?;
            return Ok(Page {
                html,
                base: base.or(Some(url)),
            });
        }
        let bytes = std::fs::read(target).map_err(|e| format!("{target}: {e}"))?;
        if bytes.len() as u64 > MAX_PAGE_BYTES {
            return Err(format!("{target}: larger than {MAX_PAGE_BYTES} bytes"));
        }
        let html = String::from_utf8(bytes).map_err(|e| format!("{target}: {e}"))?;
        Ok(Page { html, base })
    }

    /// Fetches the linked stylesheets among `refs`. Returns the sheets and,
    /// for each `<link>` that could not be loaded, its `href` and the reason.
    pub fn stylesheets(
        &self,
        page: &Page,
        refs: &[StylesheetRef],
    ) -> (Vec<Sheet>, Vec<(String, String)>) {
        let mut sheets = Vec::new();
        let mut failed = Vec::new();
        for r in refs {
            let StylesheetRef::Link { href, .. } = r else {
                continue;
            };
            if sheets.iter().any(|s: &Sheet| s.source == *href)
                || failed.iter().any(|(h, _): &(String, String)| h == href)
            {
                continue;
            }
            let Some(base) = page.base.as_ref() else {
                failed.push((
                    href.clone(),
                    "no base URL for a local file; pass --base-url".to_string(),
                ));
                continue;
            };
            let url = match base.join(href) {
                Ok(u) if matches!(u.scheme(), "http" | "https") => u,
                Ok(u) => {
                    failed.push((href.clone(), format!("unsupported scheme in {u}")));
                    continue;
                }
                Err(e) => {
                    failed.push((href.clone(), format!("cannot resolve against {base}: {e}")));
                    continue;
                }
            };
            match self.get(&url, Kind::Style, Some(base)) {
                Ok(css) => sheets.push(Sheet {
                    source: href.clone(),
                    css,
                    media: None,
                }),
                Err(e) => failed.push((href.clone(), e)),
            }
        }
        (sheets, failed)
    }

    /// A GET, repeated on transfer errors, 429 and 5xx.
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
                        let max = match kind {
                            Kind::Document => MAX_PAGE_BYTES,
                            Kind::Style => MAX_SHEET_BYTES,
                        };
                        return resp
                            .body_mut()
                            .with_config()
                            .limit(max)
                            .read_to_string()
                            .map_err(|e| format!("{url}: {e} (limit {max} bytes)"));
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
