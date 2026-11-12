---
name: viggo-reflect
description: Compatibiliteitsnaam voor de reflect-route; data-only blijft lezen zonder impliciete dailywrite.
---

# Reflectie — compatibiliteitsroute

Gebruik de actuele `reflect`-procedure voor de gekozen periode en vraag. Lees eventuele `.agents/skill-adapters/reflect.md` uitsluitend vanaf de expliciet gekozen doelroot voor persoonlijke context-/opslagbindingen. Een data-only verzoek blijft read-only; een reflectie schrijft niet stilzwijgend in een menselijke daily.

Deze alias maakt geen eigen coachmodel of opslagcollectie. Nieuwe callers gebruiken `reflect`; behoud deze naam zolang oude callers haar nodig hebben.
