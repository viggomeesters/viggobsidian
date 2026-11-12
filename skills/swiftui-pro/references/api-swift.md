# API, Swift and localization

Every recommendation is subject to the actual platform, deployment target and installed SDK. A warning-free build alone does not prove an API is current; inspect declarations or Apple documentation when necessary. An older API needed for compatibility is not automatically a defect.

- Prefer supported `foregroundStyle`, `tint`, shape clipping and builder forms of overlay/background. Do not mechanically replace `cornerRadius`, `onChange`, tabs or toolbars with unavailable overloads. Preserve `@ViewBuilder` on toolchains that use it.
- Use native Foundation format styles for user-visible numbers, dates and lists. Preserve fixed serialization formats and locale-independent IDs. Use localized date templates if DateFormatter is necessary.
- Interpolate localized Text rather than concatenate Text/string fragments. Follow the existing strings/stringsdict/String Catalog workflow; use the owning resource bundle in packages/frameworks. A runtime String converted to a localization key is not automatically extracted for translation.
- Preserve the app's chosen UI language. One app’s language does not determine another app’s language. The frontend skill owns the consumer-copy deletion pass.
- Prefer async/await for new asynchronous work when supported. Preserve callback/DispatchQueue integrations if the repository relies on their ordering or callback semantics; do not replace them solely to satisfy a style rule. Check actor isolation, cancellation, stale responses and task ownership.
- Avoid detached tasks for work requiring inherited actor/context or structured lifetime. Do not block the main actor with file scans, audio processing or expensive transformations.
- Handle recoverable errors with a meaningful action/state; printing them is insufficient when the user's requested operation failed. Force unwrap/try requires an actually unrecoverable invariant.
- Use platform-native types at framework boundaries. Preserve CGFloat where required and existing collection/storage contracts. File splitting and syntax preferences alone do not justify unrelated refactors.
