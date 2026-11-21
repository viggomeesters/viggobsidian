# Task selection core

Prepare the title and request for a task from a Markdown selection or from
a note at a collapsed cursor. The library maps UTF-16 offsets at valid
character boundaries, finds the note title, quotes literal selected text
with a sufficiently long fence, and keeps resolved attachment links in
first-occurrence order. It preserves line endings and the existing request
bytes so a consumer can compare them with stored receipts.

The caller verifies the source and editor context and resolves link targets.
It retains vault and destination selection, claims, source edits, publication,
receipts and recovery. This crate does not read or write files.

Title parsing uses the Work adapter's existing MIT-licensed
pulldown-cmark 0.13.6 with default features disabled. Literal Activity
escaping is reused from task-status-core. Consumers pin the exact reviewed
ViggObsidian revision and lockfile.
