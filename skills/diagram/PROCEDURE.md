---
name: diagram
description: Maak diagrammen on-the-fly vanuit een beschrijving. Genereert Mermaid, rendert via draw.io, exporteert PNG naar vault. Gebruik bij /diagram of wanneer de user een visueel diagram wil.
user-invocable: true
persona: creative
triggers:
  - "/diagram"
  - "diagram"
  - "mermaid"
  - "flowchart"
  - "maak diagram"
tools:
  - name: generate_diagram
    description: "Create a diagram (flowchart, sequence, architecture, etc.) from a natural language description or Mermaid syntax."
    parameters:
      type: object
      properties:
        description:
          type: string
          description: "What the diagram should represent (e.g. 'OAuth2 flow')."
        type:
          type: string
          description: "Optional diagram type (flowchart, sequence, er, state, class, gantt, pie)."
          enum: ["flowchart", "sequence", "er", "state", "class", "gantt", "pie", "mindmap"]
        mermaid_syntax:
          type: string
          description: "Optional explicit Mermaid syntax. If provided, used directly instead of generating from description."
      required: ["description"]
---

# Diagram

Maak een gericht diagram als Mermaid, draw.io XML of gerenderde PNG, volgens het gevraagde formaat. Een diagram maken publiceert niets.

Lees een optionele `.agents/skill-adapters/diagram.md` vanaf de expliciet gekozen doelworkspace voor lokale renderer-/stijl-/opslagroutes. De installatieplaats van deze skill kiest geen doelvault. Een ontbrekende noodzakelijke adapter of renderer blokkeert alleen de afhankelijke export of opslag, niet een bruikbare bronweergave; gebruik geen andere vault als fallback.

## Bestemming en bron

Bepaal het bestaande eigenaar-dossier en de expliciete uitvoerbestemming vóór export. Bij vaultopslag: volg `99-system/agents/agent-start.md` vanaf de actuele beheerde vaultroot; bewaar het diagram bij het bestaande werkpakket of dossier. Geen vaste globale bijlagenmap, iCloudfallback of impliciete dailywrite. Bewaar de bewerkbare Mermaid-/draw.io-bron bij een gevraagde export. Overschrijf bestaande bestanden alleen binnen de opdracht.

## Opbouwen

- Stappen en beslissingen: flowchart (`graph TD` of `graph LR`).
- Communicatie: `sequenceDiagram`; architectuur: flowchart met subgraphs.
- Data: `erDiagram`; lifecycle: `stateDiagram-v2`; objecten: `classDiagram`.
- Andere passende vormen: `mindmap`, `gantt`, `pie`.

Gebruik brongebonden labels, logische groepering en de taal van de gebruiker. Kies TD voor hiërarchie en LR voor volgorde waar dat helpt. Houd kleuren spaarzaam, tekst leesbaar en pijlen betekenisvol. Lees bij complexere visuele vormgeving de relevante regels van [frontend](../frontend/PROCEDURE.md).

## Renderen en controleren

Gebruik een daadwerkelijk beschikbare renderer. Bestaande draw.io MCP-tools kunnen Mermaid of XML openen, maar zichtbare UI-bediening vraagt eerst afstemming volgens de achtergrondwerkafspraak. Een toolnaam in oude documentatie bewijst geen beschikbaarheid.

Voor exacte vormgeving kan direct geldige draw.io XML worden gemaakt (`mxfile → diagram → mxGraphModel → root`, met rootcellen 0 en 1). Bij CLI-export: controleer eerst het geïnstalleerde executable en diens ondersteunde bediening; geef het exacte, zelf gemaakte invoerbestand en de gekozen nieuwe uitvoerbestemming mee:

```text
<drawio-executable> --export --format png --transparent --border 10 --scale 2 --output <exact-output.png> <exact-source.drawio>
```

Geen selectie van “het nieuwste bestand” uit Downloads of /tmp: dat kan andermans diagram zijn. Een Mermaidbestand is geen draw.io XML; gebruik daarvoor een bewezen Mermaidrenderer of converteer bewust.

Controleer exitstatus, het werkelijk gemaakte bestand en de gerenderde inhoud: tekst, pijlen, clipping en leesbaarheid. Is PNG gevraagd maar geen geschikte renderer beschikbaar, lever de bewerkbare bron en meld de ontbrekende export; noem het geen voltooide PNG.

## Opleveren

Geef het resultaatpad, een korte inhoudelijke beschrijving en bij vaultopslag een werkende embed waar passend. Native bestandsexport is geen bewijs van publicatie, clientloading of cloudbeschikbaarheid.
