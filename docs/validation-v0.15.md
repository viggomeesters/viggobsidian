# v0.15 validation

Issue #38 introduces an immutable Raw export inventory with closed payload formats. Event/Wiki/processing validation remains independent; invalid inventories fail closed.

Local validation: 76 checker unit tests, 31 checker integration tests and 28 updater tests pass; workspace fmt and Clippy with warnings denied pass. Independent review reproduced four initial bypasses, then additional comment, indentation, list and timestamp cases. The final parser rejects those cases while accepting valid export and log fixtures. Tests cover unchanged bytes, drift, missing files, paths, symlinks, duplicate members, operational metadata, complete record grammar, exporter tables, original-input provenance and ordinary wrapper enforcement. No GitHub Actions were used.

A separate private compatibility run checks existing consumer-bound payloads without importing authentic content into this repository. The contract preserves file bytes and paths; it neither repairs consumer data nor attests clinical completeness. Local providers still own mutations and processing claims.
