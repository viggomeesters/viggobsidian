---
name: design-md
description: Author/validate/export Google's DESIGN.md token spec files.
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [design, design-system, tokens, ui, accessibility, wcag, tailwind, dtcg, google]
    related_skills: [popular-web-designs, claude-design, excalidraw, architecture-diagram]
---

# DESIGN.md Skill

DESIGN.md is Google's open spec (Apache-2.0, `google-labs-code/design.md`) for
describing a visual identity to coding agents. One file combines:

- **YAML front matter** — machine-readable design tokens (normative values)
- **Markdown body** — human-readable rationale, organized into canonical sections

Tokens give exact values. Prose tells agents *why* those values exist and how to
apply them. The CLI (`npx @google/design.md`) lints structure + WCAG contrast,
diffs versions for regressions, and exports to Tailwind or W3C DTCG JSON.

## When to use this skill

- User asks for a DESIGN.md file, design tokens, or a design system spec
- User wants consistent UI/brand across multiple projects or tools
- User pastes an existing DESIGN.md and asks to lint, diff, export, or extend it
- User asks to port a style guide into a format agents can consume
- User wants contrast / WCAG accessibility validation on their color palette

For purely visual inspiration or a one-off artifact, use an actually available
frontend/design specialist rather than forcing a formal token-spec deliverable.
Historical specialist names do not prove those tools exist. This skill is for
the formal specification itself.

## File anatomy

```md
---
version: alpha
name: Heritage
description: Architectural minimalism meets journalistic gravitas.
colors:
  primary: "#1A1C1E"
  secondary: "#6C7278"
  tertiary: "#B8422E"
  neutral: "#F7F5F2"
typography:
  h1:
    fontFamily: Public Sans
    fontSize: 3rem
    fontWeight: 700
    lineHeight: 1.1
    letterSpacing: "-0.02em"
  body-md:
    fontFamily: Public Sans
    fontSize: 1rem
rounded:
  sm: 4px
  md: 8px
  lg: 16px
spacing:
  sm: 8px
  md: 16px
  lg: 24px
components:
  button-primary:
    backgroundColor: "{colors.tertiary}"
    textColor: "#FFFFFF"
    rounded: "{rounded.sm}"
    padding: 12px
  button-primary-hover:
    backgroundColor: "{colors.primary}"
---

## Overview

Architectural Minimalism meets Journalistic Gravitas...

## Colors

- **Primary (#1A1C1E):** Deep ink for headlines and core text.
- **Tertiary (#B8422E):** "Boston Clay" — the sole driver for interaction.

## Typography

Public Sans for everything except small all-caps labels...

## Components

`button-primary` is the only high-emphasis action on a page...
```

## Token types

| Type | Format | Example |
|------|--------|---------|
| Color | `#` + hex (sRGB) | `"#1A1C1E"` |
| Dimension | number + unit (`px`, `em`, `rem`) | `48px`, `-0.02em` |
| Token reference | `{path.to.token}` | `{colors.primary}` |
| Typography | object with `fontFamily`, `fontSize`, `fontWeight`, `lineHeight`, `letterSpacing`, `fontFeature`, `fontVariation` | see above |

Component property whitelist: `backgroundColor`, `textColor`, `typography`,
`rounded`, `padding`, `size`, `height`, `width`. Variants (hover, active,
pressed) are **separate component entries** with related key names
(`button-primary-hover`), not nested.

## Canonical section order

Sections are optional, but present ones MUST appear in this order. Duplicate
headings reject the file.

1. Overview (alias: Brand & Style)
2. Colors
3. Typography
4. Layout (alias: Layout & Spacing)
5. Elevation & Depth (alias: Elevation)
6. Shapes
7. Components
8. Do's and Don'ts

Unknown sections are preserved, not errored. Unknown token names are accepted
if the value type is valid. Unknown component properties produce a warning.

## Workflow: authoring a new DESIGN.md

1. **Ask the user** (or infer) the brand tone, accent color, and typography
   direction. If they provided a site, image, or vibe, translate it to the
   token shape above.
2. **Write `DESIGN.md`** in their project root using `write_file`. Always
   include `name:` and `colors:`; other sections optional but encouraged.
3. **Use token references** (`{colors.primary}`) in the `components:` section
   instead of re-typing hex values. Keeps the palette single-source.
4. **Lint it** (see below). Fix any broken references or WCAG failures
   before returning.
5. **When exports are in scope**, write the requested Tailwind or DTCG
   export next to the file, preserving the project's existing convention.

## Workflow: lint / diff / export

Before execution, verify the current official specification and CLI contract,
license and repository dependency policy. Resolve a supported package version;
do not silently install or run an unpinned latest package. The examples below
show the known command shape and must match the verified installed version.

```bash
# Validate structure + token references + WCAG contrast
npx "@google/design.md@RESOLVED_VERSION" lint DESIGN.md

# Compare two versions, fail on regression (exit 1 = regression)
npx "@google/design.md@RESOLVED_VERSION" diff DESIGN.md DESIGN-v2.md

# Export to Tailwind theme JSON
npx "@google/design.md@RESOLVED_VERSION" export --format tailwind DESIGN.md > tailwind.theme.json

# Export to W3C DTCG (Design Tokens Format Module) JSON
npx "@google/design.md@RESOLVED_VERSION" export --format dtcg DESIGN.md > tokens.json

# Print the spec itself — useful when injecting into an agent prompt
npx "@google/design.md@RESOLVED_VERSION" spec --rules-only --format json
```

All commands accept `-` for stdin. `lint` returns exit 1 on errors. Use the
`--format json` flag and parse the output if you need to report findings
structurally.

### Lint rule reference (what the 7 rules catch)

- `broken-ref` (error) — `{colors.missing}` points at a non-existent token
- `duplicate-section` (error) — same `## Heading` appears twice
- `invalid-color`, `invalid-dimension`, `invalid-typography` (error)
- `wcag-contrast` (warning/info) — component `textColor` vs `backgroundColor`
  ratio against WCAG AA (4.5:1) and AAA (7:1)
- `unknown-component-property` (warning) — outside the whitelist above

When the user cares about accessibility, call this out explicitly in your
summary — WCAG findings are the most load-bearing reason to use the CLI.


## Taste / Anti-Slop Token Gate

When DESIGN.md is used to make agents build or polish frontend UI, add explicit product-native gates instead of only palette/font tokens. These rules were mined from `Leonxlnx/taste-skill` as inspiration only; do not copy its absolute style bans into every project.

Add or verify these sections in the Markdown body:

- **Semantic color roles:** name what each token does (`canvas`, `surface`, `text`, `muted`, `border`, `accent`, `success`, `warning`, `danger`) instead of inventing decorative colors. Lock one primary accent unless the brand already has more.
- **Neutral family:** state whether neutrals are warm, cool, or true gray; do not mix warm and cool grays casually.
- **Component states:** token/prose coverage for default, hover, active/pressed, focus-visible, disabled, loading, empty, error, and success states.
- **Responsive/touch gates:** minimum touch target `44px`, mobile single-column collapse below the project breakpoint, no horizontal overflow, and dynamic viewport units (`100dvh`) for full-height shells/heroes.
- **Banned generic patterns:** no automatic AI-purple mesh, no default three equal feature cards, no unchanged shadcn-style defaults, no fake precise metrics/testimonials, no filler copy such as “Scroll to explore”.
- **Data/copy honesty:** generated UI examples must use real provided data or clearly labeled placeholders; never invent customer logos, metrics, security claims, or testimonials.
- **Motion/performance:** motion must communicate hierarchy, state change, feedback, or continuity. Animate `transform`/`opacity`; respect `prefers-reduced-motion`; avoid perpetual loops unless the product context justifies them.

For existing products, local style wins: preserve current tokens/components unless the task explicitly asks for a redesign. A DESIGN.md file should make the product more coherent, not force a fashionable external aesthetic.

## Pitfalls

- **Don't nest component variants.** `button-primary.hover` is wrong;
  `button-primary-hover` as a sibling key is right.
- **Hex colors must be quoted strings.** YAML will otherwise choke on `#` or
  truncate values like `#1A1C1E` oddly.
- **Negative dimensions need quotes too.** `letterSpacing: -0.02em` parses as
  a YAML flow — write `letterSpacing: "-0.02em"`.
- **Section order is enforced.** If the user gives you prose in a random order,
  reorder it to match the canonical list before saving.
- **Resolve the real spec version before writing.** The example's alpha label
  is illustrative, not a current-version assertion; check breaking changes and
  preserve the project's supported version.
- **Token references resolve by dotted path.** `{colors.primary}` works;
  `{primary}` does not.

## Spec source of truth

- Repo: https://github.com/google-labs-code/design.md (Apache-2.0)
- CLI: `@google/design.md` on npm
- License of generated DESIGN.md files: whatever the user's project uses;
  the spec itself is Apache-2.0.
