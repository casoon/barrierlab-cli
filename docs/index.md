---
title: Overview
description: What barrierlab-cli checks, where it stops, and how this documentation is organised.
order: 0
---

`barrierlab` fetches a web page, parses its source without a browser and runs the
[barrierlab](https://github.com/casoon/barrierlab) rule set over it. The findings use the same model
and the same rule identifiers as auditmysite, astro-post-audit and liveaudit.

## What it covers

- Structure, accessible names, ARIA, forms, tables, lists, landmarks and language: the rules that
  need only the markup and the computed role and name.
- Stylesheet rules: focus outline removal, reduced motion, orientation, justified text and tight
  line height, read from `<style>` and linked stylesheets.
- Colour contrast: the cascade over those stylesheets gives each text its colour and background,
  including gradients and modern colour syntax.
- A checklist of criteria no tool can decide, reported as untested for the page.

## Where it stops

The tool reads the HTML as the server sends it. Content that scripts build later, geometry and
interaction need a browser, and so does contrast over photos. Rules that depend on them are listed as
**not run** or **undetermined**, never as passed. See [Limits](reference/limits/).

## How the docs are organised

- **Getting started**: install the binary and check a first page.
- **Reference**: command line options, exit codes and the limits of a browser-free audit.
