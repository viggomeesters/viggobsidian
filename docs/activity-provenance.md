# Authoring Event bubble provenance

The native renderer formats a timestamp supplied by the **source adapter**.
It never reads a clock, transcript, file timestamp, Git date or vault. The
existing layout checker validates placement and shape, not source truth.

Use `activity-provenance-render` (or `activity_core::provenance::render` inside
Rust) to convert bounded source facts into an Activity day and footer. Example
stdin, using fictional data:

```json
{
  "schema": "viggobsidian.activity-provenance.v1",
  "provenance": {
    "kind": "chat-message",
    "timestamp": "2026-07-08T15:09:17-07:00",
    "display_timezone": "Europe/Amsterdam",
    "author": "A. Example",
    "channel": "Chat"
  }
}
```

The response uses `viggobsidian.activity-provenance-result.v1`, with
`status: complete`, `day: 2026-07-09`, `author`, and the Markdown `footer`:

```markdown
>
> *00:09 · A. Example · Chat*
```

Use the returned day for the outer Activity day callout and the returned
author for the message callout. Append the footer after all bubble paragraphs
and any folded Details, at the same quote depth as the title. Do not stamp the
title. Invalid requests return `status: incomplete` and exit 2; do not repair
invalid source facts with the current time. Author and channel must be explicit
plain display labels (no Markdown, control characters, or stamp delimiters).

Supply a calendar timestamp with seconds (optional fraction up to nanoseconds)
and an explicit UTC offset or `Z`, not an offset-free local clock reading.
Leap-second notation is not accepted. An unknown time is represented by
`"timestamp": null` plus `"day": "YYYY-MM-DD"` when the local day is
independently known. Only this case produces `—`. A known timestamp takes no
`day` override. The configured IANA display timezone, not the host timezone
or the current offset, determines both the day and time at that instant.

| Kind | Where the factual input comes from |
| --- | --- |
| `chat-message` | Original individual message metadata. Do not substitute import time, session update time, or current time. |
| `launcher-action` | Invocation time recorded by the producer that performed the action, not a later worker's execution time. |
| `vault-contribution` | Actual contribution authored for the vault. Label its real author and channel; it is not a chat reply receipt. |

The formatter is not a source adapter. Keep the original structured facts
with the existing source/action identity and persist those exact facts,
including the source offset and display timezone, in the writer's sealed
intent before rendering. Retries must load the same intent; they must not read
the clock again or create another intent. Keep the existing engine/binding pin:
timezone-database revisions may change historical/future zone rules. This
contract creates no registration, writer, launch, or source-access authority.

## Writer integration

- Native `vault-layout --new-work-task RAW` accepts optional `provenance`
  using the same nested fields as above for `launcher-action` or
  `chat-message`. Its existing `date` must match the derived local day, so
  folder identity, Activity day and visible clock cannot disagree. The
  recorded intent digest rejects changed facts on same-package retry,
  including changed seconds within the same display minute. The adapter still
  owns cross-package/action deduplication and stable date/slug selection. The
  input is bounded to 8 MiB and duplicate JSON keys are rejected. Long or
  conflicting literal request text fits inside folded Details, outside the
  authoritative footer.
- Native [source preparation](source-preparation.md) accepts the
  `vault-contribution` kind using the explicit `source-capture-v2` format and
  checks replay of the original preparation before creating the next
  postimage; v1 remains the legacy byte-compatibility route.
- Previously pinned requests remain compatible, and historical notes are not
  rewritten. This upstream change does **not** activate Claudian imports,
  Ruycast launcher capture, or local vault contribution providers. Their owners
  must obtain their actual source facts, persist intent and call the native
  formatter or writer route. The public tests use fictional facts only.

### Claudian: one human message with no reliable time

`claudian-activity-append` prepares a request for an **existing** guarded
Activity-append writer. It does not read a Claudian session, edit a vault, or
authorize a destination. Use it only when the caller has the original human
message, its stable provider/session/message IDs, its actual author, and a
separately evidenced local calendar day. Do not turn a session-level timestamp,
resume/turn start, UI display fallback, or import time into message metadata.
When neither original message time nor a reliable local day is known, preserve
the source outside dated Activity via the existing source-capture path.
Aggregated transcripts and assistant responses are not accepted by this
fallback; a response import must establish its separate provenance and role.

Fictional stdin example (the preimage must be the exact selected note bytes):

```json
{
  "schema": "viggobsidian.claudian-activity-append.v1",
  "vault_root": "/fictional/vault",
  "skill": "vault-project-refresh",
  "path": "20-projects/example/notes/owner.md",
  "preimage": "---\ntype: note\ncategory: knowledge\n---\n\n# Fictional owner\n",
  "message": {
    "provider": "codex",
    "session_id": "example-session",
    "message_id": "example-message",
    "role": "human",
    "body": "Original fictional input.",
    "day_basis": "human-confirmed-message-local-date",
    "provenance": {
      "kind": "chat-message",
      "timestamp": null,
      "day": "2026-11-13",
      "display_timezone": "Europe/Amsterdam",
      "author": "Example Person",
      "channel": "Claudian / Codex"
    }
  }
}
```

Keep original message bytes, exact structured intent and validator binary pin in
private source storage before rendering. The original provider/session/message
IDs are case-sensitive and must be reused byte for byte on resume. The command
does not connect to Claudian or attest that the supplied IDs are authentic.
`day_basis` is an assertion made by
that source owner: `original-message-local-date` or
`human-confirmed-message-local-date`. The command cannot verify that evidence.
Target an already resolved owner; the example does not select a vault or
create a package. Existing writer scope still applies.

On `complete`, use `result.request` as the existing `vs plan` request. Inspect
that exact trusted plan with `vs inspect` using the same root. Submit the
original input and the inspection as `{"schema":
"viggobsidian.claudian-activity-inspect.v1", "input": <original input>,
"inspection": <vs inspect result>}`. The command recomputes the recipe from
the original input; it does not trust a derived recipe. Only `verified`
matches the expected root, single path, writer goal, preimage and postimage.
Apply **that same**
reviewed plan with the existing writer, not a copy with different changes.
Use the writer's normal recovery for an interrupted apply. Restarting from a
new plan is not recovery or a substitute for inspection. For a retry after a
completed apply, read the existing owner again and resubmit the same message;
`reused` returns without a writer request. An existing same-ID bubble with
different facts or bytes is refused. This is not cross-note deduplication.

The message is literal inside folded Details, with the canonical `—` footer
outside Details and no header stamp. The command appends under the existing
same day, starts a later day, or refuses a new source dated before the current
Activity tail; it does not reorder historical entries. The bounded fallback
accepts only LF text, up to 1 MiB of note both before and after, 256 KiB of original message,
and 4 MiB per input. An incomplete request exits 2 without a write request.
The legacy renderer and writer interfaces are not changed.

## Implementation and dependency boundary

New rendering, native CLI, and tests are Rust. Jiff `0.2.39` (MIT or
Unlicense) provides checked calendar arithmetic and IANA conversions;
`jiff-tzdb` `0.1.9` (MIT or Unlicense, public-domain IANA data) is pinned
in both lockfiles. Enable only `std` and `tzdb-bundle-always` with defaults
off for repeatable native conversion without host zone files/network/clock.
The formatter is native-only; the separate Activity WASM text checker has no
new timezone dependency. Reimplementing calendar and DST rules or resolving
them differently in each writer would add avoidable calendar and replay risk.
