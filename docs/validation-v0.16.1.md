# v0.16.1 validation

Issue #43 adds exact-write-set path support for complete reading-list producer manifests and closes hidden-manifest directory aliases. Only known manifest leaf files are allowed; ordinary hidden paths, traversal, directories and symlinks remain refused. Native producer inventory and content validation remains with the source provider.

160 local workspace tests pass (78 checker unit, 31 checker integration, 51 updater); fmt and Clippy with warnings denied pass. Independent path review reproduced accepted producer leaves and rejected lexical, directory and symlink cases. Permanent tests include repeated hidden basenames, mixed hidden parents, real directory targets, dangling leaf symlinks and an ancestor symlink. No GitHub Actions, authentic source data or local provider configuration were added.
