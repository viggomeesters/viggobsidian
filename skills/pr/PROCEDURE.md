---
name: pr
description: Review a selected pull request against its specification, repository standards and concrete change risks without implicit posting or merging.
license: MIT
---

# Pull request review

Resolve the exact PR from its number, URL or the current branch. Use the existing repository integration or CLI to read metadata, base/head identities, full selected diff and relevant caller/test context. Do not publish comments, approve, request changes or merge merely because review was requested.

Pin the comparison and account for every changed file relevant to the request. Read the originating issue/specification and repository instructions. Apply [quality-audit](../quality-audit/PROCEDURE.md), keeping specification acceptance and engineering standards distinct. Select relevant perspectives instead of requiring every lens on every small change.

For a high-risk or explicitly thorough review, independent passes can improve confidence when delegation is available and authorized. Give each reviewer the same pinned scope and evidence without steering them to an expected finding. Preserve existing access and side-effect boundaries; no bypass-permission mode. If independent review is required but cannot run, report the limitation rather than label a self-review independent.

Deduplicate by the actual defect, affected code and failure condition. Agreement increases confidence, not severity automatically. Keep a useful unique finding with its evidence and uncertainty. Rank by concrete impact; distinguish blockers from speculative concerns.

Return the PR identity, reviewed base/head/diff scope, findings, actual validation and remaining uncertainty. Cite tight locations and explain how a user or caller can reach the defect. If clear, say that and name the strongest evidence checked. A later request to post or merge uses the repository's own authorization and workflow; preserve the review record rather than equating local analysis with a submitted GitHub review.
