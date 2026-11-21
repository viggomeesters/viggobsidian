# Task status core

The six launcher status keys map to their existing Markdown labels. The
Work intent supports a bare key and an object with status and reason.
The encoder preserves the compact, reason-first JSON used in existing
request hashes. The decoder retains the existing JSON and validation
behavior. Work Activity text normalizes and escapes the displayed reason
while the wire retains the original input.

This library takes strings and returns values or display text. Vault selection,
private status policy, editor eligibility, identity, claims, validation against
real notes, writes, receipts and recovery belong to the existing adapters.
Consumers pin the exact reviewed source revision in Cargo.toml and Cargo.lock.
