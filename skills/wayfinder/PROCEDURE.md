---
name: wayfinder
description: 'Find a route through a large uncertain effort: clarify the destination,
  maintain a durable decision map with its existing owner, and resolve the next ready
  decision. Works independently of Go, repositories and issue trackers. Use for wayfinder,
  wayfind or multi-session planning.'
license: MIT
metadata:
  source: https://github.com/mattpocock/skills/tree/main/skills/engineering/wayfinder
  source_commit: f3fc5632f401156837ee3872f14fe33ccf1024ea
  hermes:
    tags:
    - planning
    - decisions
    - discovery
    - multi-session
    related_skills:
    - interview
    - brainstorm
    - grill-with-docs
    - research
    - prototype
  version: 1.3.0
  author: Viggo Agent Skills (adapted from Matt Pocock's wayfinder)
  user-invocable: true
  triggers:
  - /wayfinder
  - $wayfinder
  - wayfinder
  - wayfind
  created_by: agent
---

# Wayfinder

Find the route through an effort too uncertain or large for one session.
Name the destination, map the decisions and resolve the next ready decision.
Produce decisions and a planning handoff; building the destination is separate.
If the route is already clear, give the next planning/execution step without
inventing a decision map merely for ceremony.

## Independent ownership

Wayfinder is selected through brainstorm, including explicit “wayfinder”/“wayfind” requests. It does not require Go, `.go`, a repository,
an issue tracker or a workflow installation. Never adopt/bootstrap a workflow,
create `.go`, or call a Go router simply to make wayfinding possible.

Read the actual owner's instructions and current decisions. Select storage from
that authority, not from the presence of an old directory or obsolete instructions:

- **Existing vault project/dossier:** keep the map and answers there through its
  managed writing procedure. In this vault read [vault-task](../vault-task/SKILL.md)
  for an execution task or [vault-project-refresh](../vault-project-refresh/PROCEDURE.md)
  for a bounded project update. Preserve human edits/dailies.
- **Existing repository plan:** preserve its existing GitHub Issue and design
  owner. Repository implementation follows Issue → PR → merge with local checks
  and independent review. Retired Go-state is not a planning backend.
- **Declared tracker:** use it only when already designated or explicitly requested;
  no new labels, accounts or remote publication as incidental setup.
- **No known durable owner:** continue useful research/interview; ask for the
  destination of the map before writing. Missing storage does not block conversation.
  An explicitly read-only request produces a conversational preview.

Invocation authorizes the bounded map writes at a proven existing owner, subject
to its writer contract. It does not authorize implementation, external messages,
publication or broader setup. Across repositories designate one map owner;
implementation tasks stay with each repository's own authority. Do not mirror
competing decision state or create a global queue.

## The map

Reuse an existing map or plan; create only the smallest artifact the owner needs.
A Markdown note with decision entries and links is sufficient. Separate notes or
tracker tickets help only when decisions need their own evidence or sessions.

Keep these meanings, adapted to the owner's format:

- **Destination:** one or two lines describing the outcome this map clarifies.
- **Context:** confirmed constraints, relevant sources and standing preferences.
- **Resolved decisions:** short gist and pointer to each canonical answer.
- **Ready decisions:** precise open questions whose prerequisites are settled.
- **Dependent decisions:** precise questions awaiting named prerequisites.
- **Not yet specified:** in-scope uncertainty not yet expressible as a precise question.
- **Out of scope:** explicit boundaries; do not turn them into future tasks silently.

Refer to decisions by descriptive names, with IDs/paths inside the links. A question
can be precise even when it is blocked; that belongs under dependent decisions,
not unspecified fog. Each answer has one canonical location. The map indexes it.

## Chart the route

1. Read existing owner context; distinguish settled decisions from unproven results.
2. Use [decision-conversation](../_bundle/contracts/decision-conversation.md) to clarify missing destination choices; load [interview](../interview/PROCEDURE.md) only for an explicitly requested continuing interview.
   Inspect domain/docs when they determine the meaning of an answer.
3. Explore breadth first: user journey, scope, acceptance and material uncertainties.
   Ask only ready human decisions; do not deep-dive a branch before seeing the rest.
4. Save the map at the selected owner, including prerequisites and next decision.
   Create entries before wiring references; use actual IDs/links, not placeholders.
5. Validate through the owner contract and report the next ready question.

Charting may span several conversation turns. Wait for required human answers;
do not interpret your own proposed destination as agreed. Do not execute product
work while charting. No subagents unless the user explicitly requests delegation.

## Work the map

Load the destination and compact map, then zoom into the named or next ready
question. Resolve one decision per session by default; a user-requested continuing
interview may resolve several through successive answered rounds.

Choose the needed approach:

- **Research:** establish a missing fact from local or primary external evidence.
- **Prototype:** make a bounded artifact the human can judge; respect its scope.
- **Interview:** ask the human for the product, domain or trade-off judgment.
- **Prerequisite action:** perform only an authorized action needed for a decision.

Use a claim only if the owner supports/requires it; Markdown notes do not need an
invented lock service. Re-read relevant content before writes. Record the answer,
source and resolution state without replacing concurrent edits. Human decisions
resolve only from the human's answers. A prototype without their judgment stays open.

After resolution update dependencies, sharpen newly precise questions and remove
that uncertainty from unspecified fog. Preserve discarded decisions with their
scope rationale rather than pretending they were completed. Validate the changed
artifact; report what is resolved and the next frontier.

## Completion

The route is clear when the destination and non-goals are accurate, no consequential
in-scope decision/fog remains, and another agent can plan without inventing product
or architecture choices. Hand off a compact destination, chosen approach, bounded
work units, acceptance, verification and source pointers to the owner's normal
planning/execution route. Go is optional where explicitly applicable; vault-task
or a plain bounded task contract are equally valid. No forced recommendation CLI,
workflow gate, release or implementation follows from completing the map.

## Provenance

Adapted from Matt Pocock's MIT-licensed
[wayfinder](https://github.com/mattpocock/skills/blob/f3fc5632f401156837ee3872f14fe33ccf1024ea/skills/engineering/wayfinder/SKILL.md).
Tracker setup, automatic research agents/branches/pushes and Go-only storage are
not imported. License: `../../licenses/matt-pocock-skills.txt` from this directory.

## Optional local adapter

If the user has explicitly selected a target root, read its optional `.agents/skill-adapters/wayfinder.md` for owner-specific storage/provider conventions. Resolve that file from the target root, never from this skill's source checkout. Local context is not copied into the public decision method. The source license and upstream attribution remain part of the distributed generic procedure.
