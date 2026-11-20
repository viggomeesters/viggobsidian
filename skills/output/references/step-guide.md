# Step-by-step guide

Use this for a how-to that people follow one step at a time and may also want to read in full or paste into a mail. Keep one source and produce one file with two views; never maintain a separate mail copy by hand.

**Source.** A title, an optional cover (two or three recognisable product marks, no other text), and ordered steps. A step has a short action title, a body, optionally one screenshot with the places to click, and optionally help text for when it fails. Put background, prerequisites and the keyboard hint on the second step, not on the cover. Use the real product marks and real screenshots; do not draw lookalikes. Redact secrets in screenshots unless the owner confirms they are dummy or revoked, and never publish real credentials.

**Per-step view.** One step per screen, left to right. A subtle bottom bar with previous, a step selector with count, and next; no large primary "next" button. Top right: a toggle to the all-steps view and fullscreen. Arrow keys, Home/End, `#slide-N` deep links, visible focus, and a no-script fallback that stacks all steps. Mark the click target on each screenshot with a red box.

**All-steps view.** One scrollable document in the same file. It is the paste-safe variant and is built differently from the screen view, because mail clients drop scripts, SVG, `<details>`, CSS classes and overlay boxes:

- table layout with inline styles, a fixed width of about 640 px, an explicit light colour scheme and `bgcolor` where a background matters;
- raster images only; burn the red click boxes into the image pixels instead of overlaying them;
- cover marks rendered to one image; no icon fonts;
- a copy button that selects the view and copies it as rich text, plus a plain-text fallback file with every step, command and link written out.

**Brand.** Accent colour and logo are inputs from the selected adapter. Without them use a neutral accent and no logo.

**Check.** Run the local plan and check gates on the HTML (a browser test with scripts off at desktop and phone width where available), look at both views rendered, and read the text fallback. A browser render is not proof of how a mail client or its viewer displays the paste; say so, and let the owner paste to themself first. Sending the mail stays with draft and needs its own authorization.
