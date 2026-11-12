---
name: interview
description: "Combined interview, grilling and wayfinding: reach shared understanding, challenge weak assumptions and map the next decisions. Use for interview, interview me, /interview or clarifying where a product, plan or idea is going. Scale the depth to the problem."
metadata:
  triggers:
  - /interview
  - $interview
  - interview me
  - interview mij
  - onduidelijkheid
  - eerst een paar vragen
---

# Interview

Read `.agents/skill-adapters/interview.md` from the explicitly selected target root when present for local conversation, provenance and storage bindings. Never infer the content owner from this source root. Missing storage authority blocks a dependent save, not an ordinary interview from supplied context.

An explicit interview requests an iterative conversation, not just a one-off
ambiguity gate. Start the interview in the current turn after reading relevant
context. Questions should improve shared understanding of the user's outcome,
not defend an inherited implementation plan.

## One entrance, three complementary techniques

The interview mode is selected through brainstorm; the user does not need to invoke three
skills. Adapt depth to the uncertainty within one continuing conversation:

- **Understand (interview):** establish the real job, desired outcome and meaning
  of terms; listen before recommending. Convert broad wishes into scenarios.
- **Challenge (grill):** test material assumptions against evidence, alternatives,
  failure cases and trade-offs. Be candid; distinguish a weak assumption from
  the user's settled preference. Critique the idea, not the person.
- **Orient (wayfind):** maintain destination, resolved choices, ready/dependent
  questions, remaining fog and non-goals. For a small decision this fits a brief
  recap; for a multi-session effort persist a map with its existing owner.

These are techniques, not three mandatory consecutive ceremonies. A clear small
request gets a short interview. A shaky plan needs more challenge; a broad
uncertain project needs more map. Choose the next question by its effect on the
route, not a fixed phase order. Research facts yourself before asking for choices.

Read the relevant portion of [grill-me](../grill-me/PROCEDURE.md) when consequential
assumptions need adversarial scrutiny, and [wayfinder](../wayfinder/PROCEDURE.md) when
maintaining a multi-session decision map. Apply their technique within this
interview; do not recursively invoke interview through wayfinder. The explicit
owner, scope and authorization take precedence over a specialist's generic
Repository handoff. No automatic workflow adoption, implementation or publication.

## Inspect, ask and converge

Read [decision-conversation](../_bundle/contracts/decision-conversation.md) for the shared context, ready-question frontier, answer binding and completion rules. An explicit interview starts the next meaningful question in this turn and waits for its answer; do not replace the conversation with a one-off assessment.

## Ownership and handoff

Interview does not require a repository, tracker or `.go`. Read-only/advice-only
requests stay conversational. Preserve agreed decisions at the existing owner
using its authorized writing route; a vault project uses managed vault skills.
Do not create a new workflow or overwrite human notes just to store an interview.

For critique, use `brainstorm` with the internal `grill-me` procedure;
The wayfinder mode remains available through brainstorm for map work. The combined interview
finishes with a shared destination, tested choices and the next ready action.
Planning is not implicit permission to implement or publish the destination.

## Provenance

Personal adaptation of Matt Pocock's MIT-licensed
[grilling](https://github.com/mattpocock/skills/blob/f3fc5632f401156837ee3872f14fe33ccf1024ea/skills/productivity/grilling/SKILL.md),
inspected at that commit. Decision-tree/frontier interviewing is retained;
automatic delegation and an extra universal confirmation gate are not imported.
License: `../../licenses/matt-pocock-skills.txt` from this skill directory.
