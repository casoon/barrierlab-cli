---
title: Quickstart
description: Check a live page or a saved file and read the result.
order: 2
---

## Check a page

```sh
barrierlab https://example.com
```

The tool fetches the page and its linked stylesheets, then prints a verdict, three counts and the
findings in three groups:

- **Failed**: the markup proves a violation.
- **Needs review**: a suspicion that only a person can decide.
- **Check by hand**: criteria that no tool can decide for this page.

## Check a saved file

A page behind a login or bot protection can be saved from the browser and checked from disk:

```sh
barrierlab page.html
```

## Use it in CI

```sh
barrierlab --json https://example.com > report.json
```

The exit code is `1` when at least one rule failed, `0` otherwise and `2` when the page could not
be fetched. See the [command line reference](../../reference/cli/).
