# Accessible native UI

Use the existing visual language and tokens. Native system text styles, semantic foregrounds, controls and layout should do most of the work. The frontend skill owns shared visual direction, copy and the final deletion pass; this file supplies native checks.

- Give buttons and menus meaningful text labels even when visually icon-only. Native `Button`/`Menu`/`Label` normally preserve keyboard and VoiceOver semantics better than a gesture on an image. Existing explicitly labeled custom controls can be valid; inspect their actual accessibility behavior.
- Convey meaningful states with text, shape or symbols as well as color. Preserve separate domain meanings: a headache severity color is not a diagnosis, and a recording daemon's existence is not active capture.
- Check focus order, visible focus, Return/Escape behavior, shortcuts and window reopening. Escape/closing a window must follow the app's existing action semantics, not accidentally stop a recording or timer.
- Respect Reduce Motion, Reduce Transparency and contrast settings. Keep animation purposeful and interruptible; use monospaced digits for changing timers without animating the whole window each second.
- Use semantic/scalable typography where supported. Check long localized labels and larger text settings supported by the platform. iOS touch targets and Dynamic Type rules must not be blindly imposed on macOS menu rows.
- Prefer system Label/LabeledContent/Form and supported empty states when they fit. Avoid forcing a new empty-state API on an older target. Show a concrete recovery action for a real failure.
- Health logging should remain readable in a quiet, dim appearance. Preserve the selected theme; no bright transition, flashing, obligatory motion or extra prompts introduced by a technical cleanup.
