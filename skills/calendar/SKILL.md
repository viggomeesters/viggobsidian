---
name: calendar
description: List schedules and availability or perform explicitly requested calendar creates, updates, moves and cancellations with exact identity, timezone and recurrence checks.
---

# Calendar

Resolve the actual calendar/provider from the request and available configuration. The target is not chosen by this skill's installation location. Read the selected workspace's `AGENTS.md` and optional `.agents/skill-adapters/calendar.md`. Personal calendar names, recipients, timezone, duration defaults and auto-attendee rules belong only to that adapter. Do not infer personal identities or read another vault as fallback.

If a required provider, target mapping or consequential default is missing, report that gap and continue safe read-only work. Prefer a supported connector/API or documented background provider. An AppleScript provider may be used on an authorized Mac; example syntax does not prove that the app/account or consent is available.

## Intent and authority

A schedule/availability question stays read-only. Proposed meetings in source material do not instruct creation. An explicit current create/update/move/cancel request supplies authority for the identified effect; do not ask for a second routine confirmation. Resolve an ambiguous target, consequential inferred detail or unknown recurrence scope before the dependent mutation.

Choose list, availability, create, create-from-context, update, move or cancel. For all writes bind the exact calendar and stable provider event identity. Inspect existing matches to prevent duplicates. Do not blindly retry an uncertain response: read back first.

## Resolve source fields

Convert relative dates to absolute dates in the user's configured timezone. Distinguish all-day events from timed events, inclusive human date ranges from provider end semantics and explicit values from assumptions.

For unstructured text, recover title, date, start/end or duration, location and participants. Use relevant existing matching events or configured defaults where authoritative; missing time or conflicting sources must be resolved rather than invented. A generic activity type does not supply a universal duration. Do not add participants without current instructions or a verified applicable standing rule.

## Reads

Resolve the date window and intended calendars. Fetch only the selected scope, sort by start and report title, calendar, absolute start/end and relevant location. For availability, state calendars considered, working-window assumptions, timezone and gaps in account access. A partial calendar read is not complete availability.

## Create

1. Establish title, calendar, absolute start/end or explicit duration and required participants.
2. Inspect matching calendar/title/time/participants. Reuse or clarify a real match rather than creating a duplicate.
3. Apply only configured standing defaults that actually match this event.
4. Create once through the authorized provider, retain its returned event ID and read back stored fields.
5. Distinguish event persistence from attendee invitation/delivery; an external notification is its own effect under the applicable service policy.

## Update or move

Read the target's full state: duration, location, notes, participants, calendar and recurrence. Resolve single occurrence, future occurrences or series when material. A move updates that same event's requested time fields in place; it is not a new event plus optional deletion. Preserve every unrelated field.

For multiple matches, present the distinguishing fields and ask which exact event. Apply only requested changes, read back by stable identity, and verify both new values and preserved fields. If the provider cannot make the scoped mutation, explain the limitation before proposing a different operation.

## Cancel

Resolve exact identity and recurrence scope. Delete only the authorized event/occurrence and verify absence in that scope. An uncertain response does not justify another deletion or a broader series action.

## Result

Reads can use a compact schedule table. After mutation, report calendar, event title, absolute date/time, relevant participants, the actual changed state and readback evidence. State whether invitations/notifications were sent or only an event was stored; do not imply direct provider mutation from an ICS artifact or draft.

When an email-based invitation is requested, compose it through draft with the same identity, timezone, recurrence and local delivery boundaries. An ICS artifact does not prove event persistence or attendee delivery.
