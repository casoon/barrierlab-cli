# Changelog

All notable changes to this project are documented in this file. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- **Audit:** `barrierlab <url|file>` fetches a page, parses it without a browser and runs the
  structure, semantics and stylesheet rules of `a11y-rules`.
- **Output:** text report through runemark, or `--json` with the `a11y-report` model; exit code 1
  when a rule fails.
- **Contrast:** styles are computed from the page's stylesheets with `style-cascade`; contrast is checked where colours can be determined and reported as undetermined otherwise.
- **Fetching:** browser-like request headers, retries with back-off, `--bot` and `--user-agent`.
