---
name: flutter
description: Develop and verify Flutter/Dart behavior using the target repository's real platform, dependency and architecture constraints. Load project-specific product context only from its selected owner.
---

# Flutter development

Read the target repository's instructions, `pubspec.yaml`, target-platform configuration, entry point, state/storage architecture and relevant tests before proposing changes. A folder name or example from another project is not an architecture contract. Preserve the existing stack; a Flutter task does not imply moving, bootstrapping or rewriting the repository.

If present, read `.agents/skill-adapters/flutter.md` from the explicitly selected target root for project context, platform requirements, build setup and permitted side effects. Product-specific examples and historical dependency versions stay in that local owner. A missing required platform/credential/provider prevents dependent integration or release, not unrelated source inspection. Never load a personal health-app context as a default for another app.

## Implement in the existing architecture

Keep feature, model, state, service and UI boundaries consistent with the repository. Preserve data identity, offline behavior, migrations and server authorization where affected. Prefer immutable data and clear serialization at service boundaries when those are established patterns. Confirm actual dependencies before applying Riverpod, Drift, GoRouter or Supabase-specific conventions; none is required merely because this procedure mentions Flutter.

For a connected backend, verify authentication flow and server-side access control against the current service contract. Client-side filtering alone is not row-level authorization. Do not publish credentials or production data, change authentication settings, or migrate live storage as an implementation side effect. Reproduce with fixtures or the supported test environment.

UI work follows the available frontend/platform specialist and the product's existing design language. Check safe areas, text scaling, semantic labels, contrast, appropriate touch-target sizes, keyboard/input behavior and representative loading/error/offline states. Haptics, language and theme are product decisions, not universal defaults. Prefer `const` where valid without treating it as proof of performance.

## Build and verify

Discover the actual SDK and configured devices. Use the smallest relevant commands supported by that project:

```text
flutter pub get
flutter analyze
dart format <changed paths>
flutter test <relevant tests>
flutter run -d <selected test device>
flutter build <authorized target>
```

Dependency resolution can modify lockfiles; inspect and retain only authorized changes. If code generation is part of the project, run its configured build-runner command and inspect generated diffs. Do not start persistent watchers, simulators or focus-taking UI work without the applicable environment permission.

Test the changed behavior, affected error paths and storage compatibility. A successful analyzer or build does not prove a user flow works on hardware. State whether runtime evidence came from a simulator, browser, emulator or physical device. Finish with project root, meaningful changed files, target assumptions, relevant check results and any concrete untested platform boundary. Do not imply a release or installation unless it actually occurred within scope.
