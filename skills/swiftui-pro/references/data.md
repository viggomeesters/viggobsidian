# Data flow and identity

- Keep UI selection/navigation state per window or scene. Share domain data through its existing owner; a global store must not accidentally couple independent windows' navigation.
- With Observation on a supported target, use `@State` for owned observable models and `@Bindable` for editable access. With ObservableObject, preserve the established StateObject/ObservedObject/EnvironmentObject lifecycle. Do not migrate a macOS 13 app merely to adopt Observation.
- Isolate mutable UI state to the main actor or the project's established serialization mechanism. Inspect default actor isolation and Swift language mode before proposing annotations. Use actors or another explicit owner for background mutable state; assess cancellation and lifetime as well as race safety.
- Declare view-owned state private unless its initializer is deliberately part of the public interface. Initialize state once. A view's reconstruction must not create replacement models, writers or expensive resources.
- Prefer projected bindings, but preserve semantics: a binding setter receives writes, including equal assignments; `onChange` receives observed changes, including programmatic ones. Moving persistence between them can change when or whether data is saved.
- Mutable list identity comes from a stable element ID, allocated when the element is stored. Array offsets, editable captions and UUIDs created during `body` replace row identity on ordinary changes. `id: \.self` is reasonable for small unique immutable values, not a whole editable record.
- Read locale/calendar/time zone from the UI environment when honoring per-view overrides. Define persistence timestamps and local-day boundaries in the app's domain contract; test midnight and DST when changing time aggregation.
- Do not put property-wrapper preferences inside an observable model and assume external preference changes will produce observation notifications. Use the existing preference owner and an explicit change path.
- Keep storage migrations separate from UI refactors. SwiftData/CloudKit are choices for a concrete project, not prerequisites for SwiftUI or vault integration.
