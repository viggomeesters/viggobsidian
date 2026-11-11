---
type: daily
created: "{{date:YYYY-MM-DD}}"
---

# {{date:YYYY-MM-DD}}

## Aangemaakt

```base
filters:
  and:
    - 'file.ctime.date() == date(this.file.name)'
    - 'file.path != this.file.path'
formulas:
  Notitie: file.asLink()
  Aangemaakt: file.ctime
  Dossier: file.folder
views:
  - type: table
    name: Aangemaakt
    order:
      - formula.Notitie
      - formula.Aangemaakt
      - formula.Dossier
    sort:
      - property: file.ctime
        direction: DESC
```
