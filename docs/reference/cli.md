---
title: Command line
description: Options, output formats and exit codes of the barrierlab command.
order: 1
---

```text
barrierlab [--json] [--bot] [--user-agent <ua>] [--base-url <url>]
           [--width <px>] [--dark] [--fail-on <fail|review>] <url|file>
```

| Option | Effect |
|---|---|
| `--json` | Prints the report as JSON, using the `a11y-report` finding model, instead of text. Every finding also carries `id`, its stable identity (`Finding::id()`): the rule and element pattern, or rule, file and start position where there is no selector. |
| `--version` | Prints `barrierlab <version>` and exits 0 without fetching anything. |
| `--bot` | Identifies itself as `barrierlab` and sends no browser navigation headers. |
| `--user-agent <ua>` | Sends this `User-Agent` instead of the default. |
| `--base-url <url>` | Base for resolving `<link href>`. Needed for a local file, which has none; without it linked stylesheets are not loaded. For a URL the page's own address is the default. |
| `--width <px>` | Viewport width for media queries. Default 1280. |
| `--dark` | Evaluates `prefers-color-scheme: dark` as true. Default is the light scheme. |
| `--fail-on <fail\|review>` | `fail` (default) exits 1 when a rule failed; `review` also when a finding needs review. |

## Fetching

By default the request looks like one from a normal browser: `Accept`, `Accept-Language`, the
`Sec-Fetch-*` navigation headers and a current browser `User-Agent`. Many servers turn away bare
clients, so this keeps ordinary pages reachable. Requests time out after 30 seconds, follow up to
ten redirects and are repeated up to three times on connection errors, `429` and `5xx`, with
back-off and a `Retry-After` of at most ten seconds.

Page and stylesheet bodies are capped (10 MiB and 2 MiB); a larger response is an error, not
truncated. A stylesheet that cannot be loaded (no base URL, HTTP error, too large) is not dropped
silently: the report lists it under `inputs` with the reason, and the contrast rules come out as
untested instead of passed.

A refusal (`401`, `403`, `429`) is reported as such; the tool does not try to get past a login or a
bot challenge. Save the page from a browser and pass the file instead.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | No rule failed. Findings that need review do not fail the run, unless `--fail-on review`. |
| `1` | At least one rule failed (or, with `--fail-on review`, needs review). |
| `2` | The page could not be read, or the arguments were wrong. |

## Output width

The text report wraps to the width in the `COLUMNS` environment variable, if it is set. Without it,
lines are not wrapped. Colour follows the terminal: none when the output is redirected, `NO_COLOR` is set
and not empty, or `TERM=dumb`.
