---
name: vault-brief
description: "Geef een actuele brongebonden briefing vanuit de gekozen vault, zonder impliciete taak- of statusmutaties."
---

# Bijpraten

Lees vóór handelen alle gedeelde contracten voor route `vault-brief` uit
`.viggobsidian/core/procedures.json` in de gekozen doelvault. Controleer de lokale
capability met `vault-layout --vault ROOT --skill-route vault-brief` en lees de
gebonden providerinstructies. Een groene check vervangt deze inhoudelijke lezing niet.

Lees het doelcontract uit [vault](../vault/SKILL.md) en de lokale `brief`-adapter. Lees de relevante actuele daily, inbox, eigenaars en canonieke taakselectie; gebruik de lokale tijdzone. Bevestigde prioriteiten gaan vóór open voorstellen. Controleer veranderlijke afspraken bij hun actuele provider. Een oude briefing of afwezige registratie bewijst de huidige situatie niet.

Vertel wat bekend is, wat aandacht verdient en welke concrete bestaande acties logisch volgen. Benoem bronleemten en beperkingen zonder ontbrekende gebeurtenissen te verzinnen. De briefing is read-only totdat een gekozen vervolgstap een write of uitvoering autoriseert. Vervolg die stap via de passende ingang en dezelfde taakidentiteit. Open geen UI en start geen runtime als stilzwijgend gevolg van deze briefing.
