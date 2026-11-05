# barrierlab-cli

Accessibility audit of a web page from its source: one command, no browser. It fetches a page and
its stylesheets, computes the styles, and runs the [barrierlab](https://github.com/casoon/barrierlab)
rule set over it.

```sh
barrierlab https://example.com
```

```text
[FAIL] barrierlab examples/page.html

  Fail: 7   Review: 6   Untested: 15

* Failed (7)
  - The image has no alt attribute. [images/alt-missing]
  - The input has no label. [forms/label-missing]
  - The link has no accessible name. [links/name-missing]
  - The text reaches a contrast ratio of 2.59:1; 4.5:1 is required for normal text.
    [contrast/text-insufficient]
```

## What it checks

- Structure, accessible names, ARIA, forms, tables, landmarks and language.
- Colour contrast, computed from the page's stylesheets (cascade, `var()`, media queries, modern
  colour syntax, gradients).
- Stylesheet rules such as removed focus outlines and reduced motion.

Findings use the same model and rule identifiers as auditmysite, astro-post-audit and liveaudit.

## What it does not do

It reads the HTML as the server sends it. Content built by scripts, layout and interaction need a
browser, and so does contrast over photos. Those checks are reported as undetermined or not run,
never as passed. Use auditmysite or axe for them.

## Usage

```sh
barrierlab [--json] [--bot] [--user-agent <ua>] [--base-url <url>]
           [--width <px>] [--dark] [--fail-on <fail|review>] <url|file>
```

For a local file, pass `--base-url` so linked stylesheets can be loaded. A stylesheet that cannot
be loaded appears under `inputs` in `--json` and makes the contrast checks untested.

The exit code is `1` when a rule failed (with `--fail-on review` also when a finding needs review), `2` when the page could not be read. Details:
[documentation](https://casoon.github.io/barrierlab-cli/).

## Install

```sh
cargo install --git https://github.com/casoon/barrierlab-cli
```

Requires Rust 1.88 or newer.

## Dependencies

The barrierlab crates (`a11y-static`, `a11y-report`, and everything they pull in) are taken from
git, pinned to one `rev` of https://github.com/casoon/barrierlab, because `a11y-static`,
`style-cascade` and `css-tokenize` are not on crates.io yet. This is temporary. Once they are
published, replace the two `git`/`rev` entries in `Cargo.toml` by registry versions, for example
`a11y-static = "0.1"` and `a11y-report = "0.27"`. Keep all barrierlab crates on one version so that
only one copy of each exists (`cargo tree -d` shows it).

## License

MIT
