---
name: help-skill
description: Show the commands and skills actually available for the user's need, distinguishing public entries, internal procedures, vendor tools and source owners.
---

# Skill help

Start with the user's practical need and the live session's declared capabilities. Use registered inventories when available; do not infer a public slash command merely because a PROCEDURE.md exists. Group or filter the relevant entries instead of dumping every internal name.

Read `.agents/skill-adapters/help.md` from the explicitly selected target root when present for the actual source/entry registries and profile discovery tools. General release entries, work-owned skills and private skills are different source scopes. A registry from one root does not by itself inventory all three; a source file or profile symlink does not prove the current session loaded it.

Distinguish public names from internal modes, aliases and retired compatibility procedures. Explain a requested old name's canonical successor without promising that its former slash command is still registered. Vendor plugins and project toolkits retain their real installation owners and are mentioned when relevant or requested.

Read-only discovery does not authorize an install, update, profile change or execution. When an inventory or capability is unavailable, state that concrete coverage boundary and give the useful entries actually known rather than inventing a complete system list.
