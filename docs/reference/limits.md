---
title: Limits
description: What a check of the page source can and cannot see, and how the report says so.
order: 2
---

The audit reads the markup the server sends. That is enough for most structural rules and not
enough for everything.

| Needs | Examples | Result |
|---|---|---|
| The markup | missing alt text, unnamed links and buttons, form labels, landmarks, ARIA | checked |
| Stylesheet text | removed focus outline, reduced motion, justified text | checked |
| Computed styles | colour contrast | checked from the stylesheets; **undetermined** over photos, behind absolutely positioned boxes, under filters or opacity |
| A moving focus | focus visibility | **not run** |
| Geometry | target size, overlap, reflow | **not run** |
| Scripts and interaction | content built by JavaScript, focus handling, dialogs | not seen |

## How styles are computed

The tool resolves the cascade itself: specificity, cascade layers, `!important`, the `style`
attribute, inheritance, `var()` and `@media` for a 1280 px light window. A selector or media
condition it does not understand (`:has()`, container queries) makes the affected values
undetermined unless a certain rule wins anyway. Gradients are checked against their colour stops,
not every pixel. It has not been compared with a browser on a larger corpus.

## Not run is not passed

A rule whose input is missing appears in the report as not run, with the reason. The summary counts
them, so a clean result never hides a check that did not happen.

## Pages built by scripts

The source of a single-page application holds little content, so the result says little about what
a visitor sees. Use a browser-based tool such as auditmysite or axe there.
