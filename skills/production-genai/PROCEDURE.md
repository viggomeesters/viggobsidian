---
name: production-genai
description: Engineer and verify production-oriented LLM, retrieval and agent workflows with measurable quality, data boundaries, operational controls and recovery.
license: MIT
---

# Production GenAI

Define the user/business outcome, risk level and measurable acceptance. Inspect the existing architecture, data, provider, infrastructure and team conventions before choosing technology. Prefer deterministic code to an agent where it suffices, and one bounded agent to several unless separate roles demonstrate value.

Build the actual chain needed by the request: source ingestion, retrieval, prompts, model/tool calls, UI, failure handling and deployment. A working demo is not production readiness. State explicitly which production responsibilities are out of scope.

## Baseline controls

Use proportional deterministic tests and versioned examples/golden datasets for model behavior. Evaluate quality, latency and cost against stable thresholds. Keep prompt, model, dataset and configuration versions reproducible. Calibrate model-based judges against human examples; do not treat a noisy trend as a hard invariant.

Trace requests across retrieval, model and tool calls using correlation IDs. Redact secrets and sensitive data from traces. Monitor relevant errors, timeouts, token/cost usage, latency percentiles, retrieval misses and tool failures. Bound timeouts, retries, rates and budgets. Design fallback/degradation or human handoff and document recovery/rollback.

Tool calls that are external, costly or destructive retain explicit authorization. Make side effects idempotent and long-running operations resumable. Test tool choice, arguments, state transitions, stopping behavior, recovery and prompt-injection resistance. Do not delegate authority decisions to untrusted model text.

## Retrieval systems

Measure retrieval separately from answer quality. Test representative queries, metadata filters, chunking, hybrid search and reranking where useful. Preserve source lineage and relevant citations. Evaluate both finding the right evidence and grounding the final answer in it.

Access control must apply to retrieval, embeddings, caches and traces; a permitted final UI does not compensate for a leaking index. Version and reproducibly rebuild corpus and embeddings. Add caching only with a clear freshness and authorization scope.

## Technology selection

Start with the supported provider-native API and current repository stack. Add an abstraction or framework only for a demonstrated missing capability such as durable workflow state or required interoperability. Check current official documentation and actual installed versions; examples in a skill are not version pins.

Evaluate data classification, region/retention, licenses, provider behavior, operational capacity, team knowledge, lock-in and expected costs. Use the existing approved release mechanism; do not install CI, cloud resources, new dependencies or paid providers as incidental setup. Local policy may explicitly prohibit a service or automation.

Add security/privacy review, tenant isolation, human approval, red teaming and stronger auditability as impact requires. A checklist is not evidence that those controls actually work.

## Deliver

Report the observed user result, test/evaluation thresholds and outcomes, tracing/privacy/cost coverage, deployment and rollback state, and material limits. Separate prototype behavior from proven production behavior and do not replace missing acceptance with a future improvement suggestion.
