# Product execution and evidence contract

The versioned [product capability catalog](../schemas/product-capabilities.v1.json)
is a declaration of reviewed engine contracts. It is not a compatibility probe,
runtime availability check, authorization grant, installed connection or successful
execution record. `viggobsidian product-info` only prints that embedded declaration
and the compiled engine name/version; it accepts no vault or provider arguments and
does not inspect roots, providers or files.

## Layers and decision order

1. **Selection and preflight.** The product selects an explicit user action and
   asks its native/local adapter to establish the real root, exact owner, pinned
   implementation, permissions, source identity and applicable native checks.
   A declared catalog entry does not satisfy any of these facts.
2. **Capture and preview.** Portable text rules consume a bounded, explicit capture
   and may produce a preview. Capture completeness, drift checks and cancellation
   belong to the host adapter. A clean portable result is limited to its named
   captured scope; it does not prove filesystem facts, writer eligibility or full
   vault validity.
3. **Execution by the existing owner.** The existing native provider or domain
   writer remains the only component that performs an authorized mutation. The
   product and a read-only engine interface neither take over claims, registration,
   package transfer nor recovery.
4. **Result validation and readback.** The owner validates its postconditions,
   reads back the exact target and returns an attributable result. A preview,
   request acceptance or process exit alone is not success.
5. **Recoverable failure.** On validation failure, drift, timeout, cancellation or
   interrupted execution, preserve the owner’s existing journal/receipt/rollback
   path. Do not infer completion or retry a write from a stale preview.

Thus **declaration != runtime availability != execution**. Compatibility is a
separate preflight conclusion bound to the selected implementation and local facts;
availability is a separate observation that an eligible interface can currently be
used; successful execution additionally requires the owner’s validated readback.

## Capability surfaces

Three shipped portable-text slices are deliberately narrow: `link-style-text`,
`wiki-layout-text` and `activity-text`. Their protocol and ruleset names are in the
catalog. They evaluate supplied text/snapshots and do not discover a vault or write.
`source-preparation-native`, `vault-layout-native` and
`package-lifecycle-native` remain native-only interfaces. The
`visible-folder-entrance-proof` entry is an unshipped bounded structure proof, not
a product capability or filesystem attestation.

## Timing and claim ceilings

End-to-end timing starts when the user action enters selection/preflight and ends
only after validated readback, or after a terminal failure/cancellation and required
owner cleanup/recovery state has completed. Capture, preview, adapter work, native
execution, validation, readback, cancellation and failure handling are all in that
interval. Report observed measurements with their concrete scope and environment;
do not turn a local rule-evaluation or writer measurement into a generic speed,
latency, uptime or completion guarantee. A deadline or cancellation signal must
produce a terminal non-success result unless the existing owner has independently
validated a completed operation.
