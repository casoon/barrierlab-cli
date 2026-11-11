# Changelog

All notable changes to this project are documented in this file. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- **Output:** `--json` adds `id` (the stable `Finding::id()`) as the first field of every finding, so hosts can compare runs and group findings without recomputing it.
- **CLI:** `--version` (`-V`) prints the version and exits; before, it was read as a file name.
- **Audit:** `barrierlab <url|file>` fetches a page, parses it without a browser and runs the
  structure, semantics and stylesheet rules of `a11y-rules`.
- **Output:** text report through runemark, or `--json` with the `a11y-report` model; exit code 1
  when a rule fails.
- **Contrast:** styles are computed from the page's stylesheets with `style-cascade`; contrast is checked where colours can be determined and reported as undetermined otherwise.
- **Audit:** `--base-url` resolves linked stylesheets of a local file; `--width` and `--dark` set the
  media-query environment.
- **Exit code:** `--fail-on review` also exits 1 when a finding needs review.
- **Input:** a stylesheet that cannot be loaded is listed under `inputs` in `--json` and in the text
  report, with the reason; contrast is then untested instead of passed.
- **Fetching:** page and stylesheet bodies are size-limited (10 MiB and 2 MiB).
- **Fetching:** browser-like request headers, retries with back-off, `--bot` and `--user-agent`.

### Changed

- **Pages:** the project site uses the current shared theme (0.12.12).
- **Audit:** parsing, stylesheet discovery (`<link media>`, `disabled`, `rel=alternate`, `<style media>`, `hidden`) and the cascade wiring come from `a11y-static`; the CLI only fetches.
- **Dependencies:** barrierlab crates come from crates.io (`a11y-static` 0.1, `a11y-report` 0.28) instead of a sibling checkout; `cargo install --git` works on its own.
- **Dependencies:** `runemark` 0.10. In a terminal, an empty `NO_COLOR` no longer turns colour off, and `TERM=dumb` does; piped output is unchanged.
- **Dependencies:** `a11y-static` 0.2; a sheet that cannot be loaded is passed as `Sheet::failed` with its reason, so the CLI no longer rewrites the report notes itself.
- **Dependencies:** `a11y-static` 0.5 and `a11y-report` 0.30: every finding now carries `help_url` (link to its rule on the barrierlab site), the start-tag end in `location.end` and the same stable `Finding::id()` as the other tools.
