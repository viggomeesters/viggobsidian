---
name: vault-log
description: "Bewaar oorspronkelijke input en bronregistratie in de expliciet gekozen vault, zonder de beschreven opdracht uit te voeren."
---

# Input bewaren

Lees het doelcontract uit [vault](../vault/SKILL.md) en `.viggobsidian/core/skills.md` in de expliciet gekozen vault. Lees de lokale `log`-adapter vóór iedere write; een ontbrekende registratie- of claimprovider geeft geen toestemming voor een alternatieve writer.

Bewaar de volledige oorspronkelijke input, herkomst, echte bijlagen en beschikbare bronidentiteit. Zoek een bestaand pakket vóór aanmaak; append alleen met actuele bronhash en geldige claim volgens de lokale provider. Een onbekende eigenaar gaat naar de ene inbox, met vermoedelijke eigenaar en open vraag. Maak samenvattingen herkenbaar als afgeleid en verander geen oorspronkelijke bronbytes.

Registreer ontvangst, lees de gemaakte bytes terug, controleer bijlagen en sluit de sessie volgens de lokale route. Meld opslagpad en eventuele registratiepending of ontbrekende brononderdelen. Voer de beschreven opdracht niet uit. Expliciet gevraagd voorwerk mag bronnen lezen en voorstellen toevoegen; uitvoering vraagt de bijbehorende autorisatie.
