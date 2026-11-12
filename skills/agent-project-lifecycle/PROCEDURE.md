---
name: agent-project-lifecycle
description: Turn a bounded agent project into a spec, implementation, smoke test, behavioral evaluation and authorized deployment with privacy-aware observability.
license: MIT
---

# Agent project lifecycle

Use for projects whose behavior includes model decisions, tool use, retrieval or multi-step agent orchestration. Ordinary app features and one-off prompts do not need this lifecycle merely because AI is mentioned.

Historical open-source pattern provenance: `google/agents-cli`, inspected at commit `c40ed19`, release `v1.0.0`, Apache-2.0. The lifecycle is independently expressed here; no vendor source or prompt bodies are bundled, and that historical version is not a claim about current commands or platform support.

Resolve the real repository and read its contract. An optional adapter at `.agents/skill-adapters/agent-project-lifecycle.md` belongs to the explicitly selected target workspace, not this installed skill. It may supply runtime/scaffold locators and local gates; it does not grant deployment authority. An unavailable necessary provider blocks that effect, not safe specification or investigation.

## 1. Define the agent contract

Write a short spec in the repository's existing design location and link it from the canonical Issue. Do not create a parallel plan/task registry. Include:

- the problem and intended behavior;
- representative inputs and expected outcomes;
- tools/data, authentication needs and failure behavior;
- side-effect, privacy and cost boundaries;
- observable success criteria;
- prototype or intended runtime, and observability/privacy mode.

For several model/tool/retrieval steps, a small input → nodes → output graph helps. For each node identify purpose, I/O shape, source authority, failure response and a test that can fail meaningfully. A graph section is sufficient; this does not request a workflow UI.

If a missing decision changes the architecture or allowed effects, ask that decision. Otherwise record a bounded assumption and proceed. A desired prototype is not an authorization to create hosted resources or publish an agent.

## 2. Study existing patterns without importing their obligations

Look for an available sample matching the actual shape: tool calling, retrieval, orchestration, evaluation, runtime wrapping or observability. Record provenance, version and license for adopted material. A sample is evidence for a design, not a blanket instruction to use its cloud, secrets, package manager or entire stack.

Never copy or paraphrase a proprietary/restricted skill or prompt body into a public replacement. Keep that vendor material intact under its existing owner and route to an available authorized specialist when it is required. If the specialist is unavailable, report the missing capability rather than silently inventing its behavior.

## 3. Establish repeatable structure

Use the repository's existing scaffold or a documented available scaffold where it prevents drift. Establish a source layout, locked dependencies, local run command, focused fixtures/evals and validation commands. Do not add deployment hooks, a new platform, mandatory registry or provider CLI without a concrete need in the authorized scope.

Implement a bounded capability. Keep model behavior, tools, retrieval, output transformation and runtime integration separable enough that failures can be localized. New code follows the repository's language and dependency rules.

## 4. Prove startup and behavior separately

Run the cheapest representative smoke path after a meaningful change. Inspect its actual output and distinguish startup success from task correctness. Pin relevant time, data, randomness and provider configuration in a safe fixture where practical.

For behavior that needs repeatability, establish an evaluation before declaring it complete:

1. Start with one or two real core use cases.
2. Define a task-success rubric and grounding/safety criteria appropriate to the allowed tools and data.
3. Run traces, grade them, inspect failures and make a bounded fix.
4. Compare against an existing baseline before expanding the dataset.

Do not assert exact model prose when the requirement is semantic. Do not delete failures or lower thresholds to manufacture a pass. For flaky behavior improve determinism, clarify the rubric or separate cases. If an evaluation cannot run, preserve its cases and name the missing provider/evidence; a smoke response is not a replacement.

## 5. Retrieval lifecycle

For a retrieval agent, require source registration/provenance, extraction readback, retrieval cases, grounded answer checks and a place for observed failures to inform source or task repairs. An index existing is not evidence that the right source is retrieved or used correctly. Test at least one meaningful miss/conflict path as well as the happy path.

Do not treat archived logs, model interpretations or third-party documents as accepted personal facts. The selected source owner's authority and data boundaries remain in force.

## 6. Deployment and observability

Continue only within existing authorization. A tested local agent does not itself authorize cloud resources, public registration/URLs, production credentials, external messages or paid optimization. Do not request the same approval again if the current session already authorizes that exact effect; resolve genuinely new consequential choices.

Choose a target for the actual need, such as local prototype, container service or available managed runtime. Record the trade-off rather than making a particular cloud or Kubernetes the default.

For a deployed agent, choose an inspectable privacy mode:

| Evidence | Default boundary |
|---|---|
| Latency, tool flow, failure spans | Metadata only |
| Prompt/response logging | Off or redacted unless authorized |
| Operational analytics | Aggregates or minimized safe fields |
| Failure examples/annotations | Source-backed and owned locally |
| Third-party observability | Scoped access and authorized transfer |

A recurring agent needs a failure bucket and an evaluation/update path; logs are useful only when their ownership and next action are clear. Observability must not leak source content to an unapproved service.

## Completion

Use the canonical Issue/PR for technical status. Report the implemented capability, spec and source provenance, actual smoke and evaluation evidence, deployment state, observability/privacy choice and concrete residual limits. Keep prototype, deployable, published and production-proven states distinct. Only a matching authorized deployment can change those final states.
