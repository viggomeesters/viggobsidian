---
name: meeting
description: Prepare a meeting, process a selected recording or transcript, or capture an explicit debrief with source identity and traceable decisions and actions.
license: MIT
---

# Meeting

Select one mode from the user's request: preparation, recording/transcript processing, or debrief. Do not search for recordings when the request supplies a debrief, and do not treat preparation as permission to schedule, invite or send messages.

For a selected target root, read its authority and provider rules and optionally `.agents/skill-adapters/meeting.md` there. The adapter owns recorder locations, contact matching, note format and supported storage writers. The installed skill's directory never selects that target. Missing provider access blocks only the dependent lookup or write, not a preparation from supplied context.

## Preparation

Read the relevant current project, participants, recent source-backed interactions and open questions. Reuse existing preparation rather than create a second dossier. Put the useful conversation topics first: known fact or issue, decision needed and a concrete question. Keep private negotiation notes separate from participant-facing material. Unknown date, identity or role remains unknown.

If the user requests a participant email, use draft/output for the actual body and correct audience; include the usable preparation itself, not only a local-file link. A personal brief remains a brief. Record only through an authorized target writer. Do not create a daily note, reminder job or invitation as a side effect.

## Recording or transcript processing

1. Pin the exact source package, recording/transcript identity, date and existing owner. Read the complete selected transcript and available matching metadata; do not silently substitute a recent similarly named recording.
2. Check whether fragments belong to one conversation. Matching participants and nearby times are candidates, not proof. Use direct recording/session evidence and contradictory metadata. Distinct participants or incompatible times remain separate; ask only when the needed identity cannot be resolved. Keep all original fragments and their offsets.
3. Preserve original transcript bytes. Treat suspected repetitions, ASR hallucinations and speaker uncertainty as interpretation issues; do not silently delete or reassign original evidence. Distinguish a readable derived transcript from the original. Never infer a speaker solely from guessed diarization.
4. Identify action candidates with their actual segment/time references, speaker and surrounding context. Confirm a real agreement rather than counting imperative verbs. Separate decisions, proposals, questions and observations.
5. Summarize the substantive outcome: concise overview, relevant topics, confirmed decisions, actual follow-up commitments and open points. Name an owner or date only when supported. Use enough actions to preserve the agreements; no arbitrary quota and no task per sentence.
6. Reuse existing tasks. Create linked actions only when requested or authorized by the target's existing processing contract. The task owner holds status; the meeting note is source evidence, not a second task register.
7. Use the selected writer to preserve the complete source package and derived result, including attachments and provenance. Re-read destination, source links and exact identities. Mark processing only after the actual output and source integrity are verified. A marker proves neither backup nor transcript quality.

Duration comes from reliable matching call/recording metadata where available. Otherwise report the recording/transcript duration and its uncertainty; it is not necessarily the whole meeting. Participant/contact enrichment needs the selected domain's authority and must not create personal entities in a work project automatically.

## Debrief

Keep the user's complete original account and factual date/participants. Distinguish their observation from a confirmed external fact, a decision and a possible next step. Use the same owner and traceable action rules as recording processing. Do not search a recorder, update contacts or send follow-ups merely because a debrief was supplied.

## Finish

Report the selected mode, actual source scope, resulting note/file paths, extracted decisions/actions, verification and material missing context. Do not claim storage when only a concept was prepared. A read-only or concept-only request stays within that scope. Scheduling, participant messages, contact updates and daily edits require their own explicit authority.
