---
title: Command line
description: Options, output formats and exit codes of the barrierlab command.
order: 1
---

```text
barrierlab [--json] [--bot] [--user-agent <ua>] <url|file>
```

| Option | Effect |
|---|---|
| `--json` | Prints the report as JSON, using the `a11y-report` finding model, instead of text. |
| `--bot` | Identifies itself as `barrierlab` and sends no browser navigation headers. |
| `--user-agent <ua>` | Sends this `User-Agent` instead of the default. |

## Fetching

By default the request looks like one from a normal browser: `Accept`, `Accept-Language`, the
`Sec-Fetch-*` navigation headers and a current browser `User-Agent`. Many servers turn away bare
clients, so this keeps ordinary pages reachable. Requests time out after 30 seconds, follow up to
ten redirects and are repeated up to three times on connection errors, `429` and `5xx`, with
back-off and a `Retry-After` of at most ten seconds.

A refusal (`401`, `403`, `429`) is reported as such; the tool does not try to get past a login or a
bot challenge. Save the page from a browser and pass the file instead.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | No rule failed. Findings that need review do not fail the run. |
| `1` | At least one rule failed. |
| `2` | The page could not be read, or the arguments were wrong. |

## Output width

The text report wraps to the width in the `COLUMNS` environment variable, if it is set. Without it,
lines are not wrapped. Colour follows the terminal: none when the output is redirected or
`NO_COLOR` is set.
