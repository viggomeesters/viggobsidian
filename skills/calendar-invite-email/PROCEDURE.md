---
name: calendar-invite-email
description: Prepare or deliver standards-compliant iCalendar invitations through the configured provider, with stable updates, timezone-safe fields and explicit delivery evidence.
---

# Calendar invitation via email

Use when an email/ICS invitation is the requested calendar effect. It is not the same as writing a Google, Apple or other calendar API event. Prefer direct provider operations when those are what the user explicitly requested and access exists.

Resolve the selected target workspace and read its `AGENTS.md` and optional `.agents/skill-adapters/calendar-invite-email.md`. That adapter owns the sender, organizer, actual recipients, account/provider locator, standing attendee rules and delivery restrictions. Never infer recipients from generic examples or this skill's source location. Read related calendar defaults only from the selected authorized context.

Sending an invite is an external email effect. Follow the actual local email policy, including a human-only-send rule where applicable. A current unambiguous request can authorize an effect, but cannot override a stricter standing rule unless the user explicitly changes that rule. Preparation does not authorize sending. Missing provider/access blocks delivery, not a useful source-bound draft where the necessary fields are known.

## Establish the event

Recover title/context, exact dates, start/end, all-day versus timed, timezone, sender/organizer and recipient set from current sources. If the source is a screenshot or received message, re-read its actual fields and decide whether the original sender or mail recipient was intended to attend; do not assume.

For updates, retain the event's existing UID and increment SEQUENCE. A title clarification or polish is not reason to send a new UID and create duplicate events. For catch-up invitations, identify the missing recipients and update scope instead of blindly resending to everyone. Apply only explicitly applicable standing defaults from the local adapter.

## iCalendar artifact

Use a documented existing provider/helper where available and verify its current argument contract. Do not invent a vanished script path or expose its environment secrets. If implementing within an authorized task, obey that repository's language/dependency rules.

The resulting ICS must include:

- VCALENDAR with VERSION:2.0, PRODID, CALSCALE:GREGORIAN and METHOD:REQUEST for an invite;
- one intended VEVENT with stable UID and UTC DTSTAMP;
- DTSTART/DTEND using VALUE=DATE for all-day or unambiguous timezone-aware/UTC timed values;
- SUMMARY, organizer address and intended attendees with the appropriate RSVP/participation fields;
- STATUS and SEQUENCE consistent with the requested create/update operation.

All-day DTEND is exclusive: a human range including its last day ends at the following calendar date. Do not turn that into a 24-hour arithmetic assumption across timezone transitions. For timed events, verify source timezone and daylight-saving conversion rather than attaching a timezone label to a naive timestamp.

Choose busy/transparent status from the user's request or a verified local default; a vacation is not universally transparent. Escape calendar text and apply RFC-compatible line endings/folding with the real implementation. The event's actual fields, not a display-only title, are the verification source.

For email transport, prepare both a `text/calendar; method=REQUEST` alternative and an `.ics` attachment fallback. The visible body should state the same title, date/time and scope. Do not claim client RSVP/add controls were tested merely because the MIME structure is valid.

## Before delivery

Run the actual provider's dry-run or equivalent draft/export path. Inspect the generated ICS/MIME and exact sender, recipients, title, dates, timezone, UID, SEQUENCE and update scope. Record filenames and inspect source-derived assumptions; do not print SMTP credentials, tokens or private environment values.

If local policy reserves email sending for the user, stop at the checked draft/ICS handoff. Do not bypass that boundary with another transport or interpret an available SMTP helper as permission.

## Readback and completion

For authorized delivery, inspect actual provider acceptance/refusal and recipient scope. State partial failures without blind retry. Retain event identity and delivery evidence with the source owner. Queue admission or SMTP acceptance alone does not prove calendar acceptance, attendee participation or recipient inbox rendering.

Report the actual outcome: prepared ICS/email, delivered invite, updated invite or direct calendar mutation. Include concrete title/date/time, relevant recipients and meaningful limits. Preserve the related source/task identity; calendar delivery does not automatically complete a broader task.
