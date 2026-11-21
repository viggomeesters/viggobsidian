# Quick capture preparation

Version 0.1.0 provides the first-line title, dated filename stem, Markdown inbox
header and human Activity message shared by Quick Note and Quick Task. The
release is a Rust source dependency. Pin the exact reviewed ViggObsidian
revision and commit the consumer's Cargo lockfile.

The writer supplies the existing identity, date, time, capture style and labels.
It retains input validation, the original input bytes, root and destination
selection, Activity section validation, vault checks, claims, writes and
recovery. The library does not access the filesystem or clock. The launcher
still decides whether to open the saved note.
