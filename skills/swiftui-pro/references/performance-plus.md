# Diagnosed broad invalidation

Use only for an explicit deep review or measured excessive updates. Reading several fields from an observable object is not itself a defect: Observation tracks accessed stored properties. A computed getter also tracks the stored inputs it reads; it does not narrow that dependency automatically.

- A stored struct is one observed dependency. If a frequently changing field invalidates unrelated expensive readers, consider separating that field or passing smaller immutable inputs. Do not convert value models to reference models without a demonstrated benefit and preserved semantics.
- A smaller observable child model can narrow updates when many views consume one part of a large model. Create and retain it at the data owner; avoid wholesale row-model recreation.
- Cache an expensive derived value only with all update paths covered. Compare update counts and UI correctness before and after. Extra model structure without a measured improvement is not a successful performance change.
- If an expensive view reads a field solely for an onChange side effect, isolate that observation in a small view/modifier when it materially reduces work. Preserve the side effect's ordering and lifetime.
- Record the representative workload, observed bottleneck, intervention and comparison. Keep profiler/build-only evidence distinct from live app responsiveness.
