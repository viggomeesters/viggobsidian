---
name: frysk
description: "[DEPRECATED → use /draft frysk] Vertaal tekst naar Fries. Referentie voor grammaticaregels en woordenboek."
persona: diplomatic
user-invocable: false
triggers:
  - "/frysk"
  - "vertaal"
  - "frysk"
  - "fries"
  - "vertaal naar het fries"

---

# frysk -> /draft frysk

This skill is deprecated and kept only for trigger routing and backward compatibility.

## Canonical Target

/draft frysk

## Action

Use `draft` with Frysk intent so identity, channel, send boundary, and wording rules stay centralized.

## Output

Report the canonical route used, whether any action was taken, and the exact command, file, or skill that owns the result. If no action is taken, state the blocker clearly.

## Stop Conditions

- Do not send translated text from this skill.
- Do not maintain a separate Frysk message workflow here.

## Migration Note

Prefer new docs, tasks, and commands that reference `/draft frysk` directly. Keep this deprecated skill small so old workflow details do not compete with the maintained path.
