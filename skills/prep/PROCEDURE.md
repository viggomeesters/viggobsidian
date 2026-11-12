---
name: prep
description: Compatibility route for meeting preparation through the canonical meeting skill.
license: MIT
---

# Meeting preparation compatibility

Use [meeting](../meeting/SKILL.md) in preparation mode with the original request. If the original request explicitly asks for debrief or transcript processing, preserve that mode instead. Do not run a historical preparation job, search recordings by default, schedule an event, create duplicate notes or send a message from this alias. This independently written router replaces no licensed source manual.
