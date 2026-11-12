---
name: brainstorm
description: Use when the user wants to shape an idea, plan, architecture, workflow, or implementation approach before building. Inspect available context, use grill-style pressure testing, and only escalate to repository planning/spike when a real execution boundary is needed.
version: 1.0.0
author: Hermes Agent
license: MIT
user-invocable: true
triggers:
  - "/brainstorm"
  - "brainstorm"
  - "brainstorm:"
  - "denk mee"
  - "maak dit scherper voordat ik begin"
  - "grill dit plan"
metadata:
  hermes:
    tags: [lane, planning, before-build, brainstorm, decision-quality]
    related_skills: [grill-me, grill-with-docs, github-work, spike, writing-plans]
---

# Brainstorm

Shape an idea, plan, architecture or workflow before building. The outcome is a clearer direction, scope, main risk and next action—not automatic implementation or task creation.

Read [decision-conversation](../_bundle/contracts/decision-conversation.md) for the shared source-first conversation and ready-decision rules.

## Choose depth

- Ordinary idea/design: reason through options here, pressure-test material assumptions and recommend a shape.
- Explicit continuing interview: [interview](../interview/PROCEDURE.md).
- Adversarial critique or stakeholder/demo translation: [grill-me](../grill-me/PROCEDURE.md).
- Large multi-session uncertainty requiring a decision map: [wayfinder](../wayfinder/PROCEDURE.md).
- Domain/docs/ADR stress test: [grill-with-docs](../grill-with-docs/PROCEDURE.md).

Choose only what the request needs. These are techniques in one conversation, not a required sequence of commands. Existing-artifact inspection belongs to audit, just-delivered repair to recheck, future improvements to next and agent-method feedback to selfimprove.

## Handoff only when requested

Read [work-handoff](../_bundle/contracts/work-handoff.md) when durable direction, a task or execution is requested. Use the actual owner: GitHub Issue and PR for repository work when applicable, managed vault work for a vault task, or a plain bounded contract. Conversation alone creates no taskstate.

For repository work, [GitHub work](../github-work/SKILL.md) selects the appropriate internal mode from the requested outcome and stop boundary. A risky technical assumption may justify [prototype](../prototype/PROCEDURE.md), only within an authorized experiment boundary.

## Result

Give the target, main risk, meaningful alternatives, recommended shape and next action. Use only fields that help this decision. Expose material unresolved assumptions; ask only the decision that blocks a real branch. Show one useful public next command when needed, not the complete repository menu. Planning does not implicitly authorize building, publishing, sending or buying.
