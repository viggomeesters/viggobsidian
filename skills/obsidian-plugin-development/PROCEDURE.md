---
name: obsidian-plugin-development
description: Develop, validate and publish an Obsidian plugin with phase-aware release and actual host/catalog evidence.
---

# Obsidian plugin development

Use for a repository with an Obsidian root manifest (`id`, `minAppVersion`) or an explicit Obsidian-plugin task. Read this procedure before implementation, preparing a version or sending the owner to the submission form. It is a conditional specialist under Go/release, not a new task engine or permission to publish. Read the selected repository instructions and `.agents/skill-adapters/obsidian-plugin-development.md` when present. Local test-vault paths, source owners, authorizations and unavailable infrastructure stay in that adapter/host instructions. Never infer them from this skill's installation path.

Read the [architecture/checklist](references/checklist.md), [failure catalog](references/failure-catalog.md) and actual current official sources linked there. Reuse the [plan/evidence outline](references/plan-template.md) in the existing Issue or repository evidence; do not create another registry or mandatory vault task. Resolve the existing ViggObsidian CLI from the approved host installation, or build the pinned reviewed ViggObsidian release with locked dependencies. `plugin-check` is read-only: it does not install dependencies, execute repository tests, submit, publish or modify the vault.

## First submission: release before registration

The official directory reads `manifest.json` on the repository's **default branch**. For a new plugin, a public exact-tag GitHub release with the standard assets must exist **before Submit/personal declaration handoff**. A draft release or tag alone is not sufficient. Do not make an unavailable pre-registration dashboard preview a prerequisite that forces the owner into a missing-release screen.

1. Inspect ownership, overlapping plugins, distribution, privacy and the actual architecture. A private repository needs a separately authorized public-source decision, not automatic exposure of its history.
2. Implement and validate the exact candidate locally and in the isolated test vault; obtain independent review. Record real local/host results, candidate commit and main.js digest in the existing evidence using the [small schema](references/evidence-template.json). These are supplied observations, not facts a checker can manufacture.
3. Merge within authority, select the exact final candidate/tag commit and publish the authorized GitHub release. Verify with `viggobsidian plugin-check --repo-dir REPO --github OWNER/REPO --candidate COMMIT --phase public-release --initial --evidence EVIDENCE.json`. It must pass before the first submission. Keep this gap within the active release operation; do not report shipped while a manifest points only to an unpublished release.
4. Only then register/submit. The owner handles personal policy/support declarations or legal agreements when required. Run the full official release review, including build reproduction. A first submission may not have an earlier source-preview report; document this explicitly instead of inventing one.
5. Publish the listing within authority, verify the visible version and perform a fresh real app-catalog install. Run `--phase catalog-install --initial --installed-dir TEST-VAULT/.obsidian/plugins/ID --evidence EVIDENCE.json`. Only this final phase can report complete delivery. A manual copy, public listing, positive build or synthetic host test is not a catalog-install pass.

## Existing plugin update

Keep the prior public version usable while preparing a candidate branch. Run the local/host/review checks and `--phase source-preview`; missing/draft release is an **expected pre-release state**, not public delivery. Run an official branch preview where available before publication. Bind that observation to the exact final release commit; after a merge changes that identity, rerun the preview or choose the already reviewed reachable source commit according to the repository contract. Never silently equate different commits.

After the final merge/tag/upload, run `--phase public-release` with source-preview evidence. Then run the full official release/build review and the real installation/upgrade acceptance in `--phase catalog-install`. The source preview never substitutes for full release/build verification.

## The checker and its limits

```
viggobsidian plugin-check --repo-dir REPO --github OWNER/REPO --candidate COMMIT --phase source-preview --evidence EVIDENCE.json
viggobsidian plugin-check --repo-dir REPO --github OWNER/REPO --candidate COMMIT --phase public-release --evidence EVIDENCE.json
viggobsidian plugin-check --repo-dir REPO --github OWNER/REPO --candidate COMMIT --phase catalog-install --installed-dir TEST-VAULT/.obsidian/plugins/ID --evidence EVIDENCE.json
```

Add `--initial` for first registration. Exit 0 means the selected phase passes; 1 means fail or pending, which the JSON distinguishes; 2 means invalid input/collection failure. `phasePassed` is not `deliveryComplete`. The checker collects Git/GitHub/anonymous download observations. Local test, runtime, official review and install-channel assertions are **supplied evidence** with explicit candidate/payload binding; it cannot certify their truth or replace review. No-evidence output remains pending rather than silently green. JSONC/extended TypeScript configuration may need a reviewed effective-library resolution; do not suppress a known library failure.

For JSONC/extended TypeScript config, record the reviewed `tsc --showConfig` result in `effectiveTypeScript` with the exact `commit`, effective `libraries` and evidence locator. This resolves an unknown library result; it cannot override a proven explicit missing library.

The first release gate rejects drafts, prefixed/wrong tags, wrong default-branch metadata, wrong dereferenced tag commit, missing or byte-different assets and a mismatched downloaded manifest. CSS is conditional on actual use. Anonymous downloads establish public availability; authenticated access to a private release does not. An installed main.js may differ only by the specifically observed exact Obsidian `\n/* nosourcemap */` suffix, not arbitrary trimming or normalization.

If the new ID is absent from the app registry after a passed/published listing, leave catalog acceptance pending with the actual attempt, reason, supported propagation window and concrete recheck date. Restore verified prior/manual files if necessary and label that channel honestly. Do not bump/rebuild for propagation alone or claim an automatic recheck unless one was explicitly requested and created. For substantive review findings repair the candidate, repeat affected checks and use a new version after a published release; preserve published assets/tags.

## Report and maintain

Report separately: tested source/host, public GitHub version/assets, official review/build result, public listing and actual Community install. Preserve untested platforms and minimum-version gaps. Update stale README/install/status wording after publication. Keep technical evidence in the Issue/repo and human decisions at their established owner. Consult the failure catalog when a recurring error appears; improve one governing check/route, not a copied checklist per incident. No retired workflows, global queue, automatic publisher or newly enabled CI system.
