---
name: tdd
description: "Use when implementing features, bug fixes, behavior changes, or refactors with test-driven development: one failing behavior test, minimal code, refactor, repeat."
version: 1.0.0
author: Hermes Agent (adapted from mattpocock/skills tdd and Hermes test-driven-development)
license: MIT
user-invocable: true
triggers:
  - "/tdd"
  - "tdd"
  - "test-driven"
  - "red green refactor"
  - "test first"
metadata:
  hermes:
    tags: [testing, tdd, red-green-refactor, behavior-tests, tracer-bullets]
    related_skills: [test-driven-development, systematic-debugging, writing-plans, grill-with-docs]
created_by: agent
---

# TDD

For an explicitly selected target, read `.agents/skill-adapters/tdd.md` when present for actual test commands and local effect boundaries. The installed skill source never selects a data/repository root. If an effect requires an absent local adapter, keep that effect pending rather than inventing a writer or test result.

## Overview

Use strict red-green-refactor for code work. The goal is not “more tests”; it is **faster feedback and less agent drift**.

Core rule:

```text
One behavior test fails -> minimal implementation passes -> refactor while green -> repeat.
```

Tests should verify behavior through public interfaces, not implementation details. Good tests survive refactors; bad tests break when internals move while behavior stays the same.

## When to Use

Use for:

- new features;
- bug fixes;
- behavior changes;
- refactors with observable behavior risk;
- agent/subagent implementation tasks where correctness matters.

Usually skip for:

- throwaway prototypes/spikes that will be deleted;
- pure docs/copy edits;
- generated boilerplate;
- config-only changes with no runnable behavior.

If a bug is already observed, read [debug](../debug/PROCEDURE.md): reproduce and understand root cause first, then write the regression test before fixing.

## Planning Before RED

Before touching production code:

1. Inspect existing tests, public APIs, docs, domain glossary, and ADRs when present.
2. Identify the public interface/capability under test.
3. List the next behavior, not the implementation step.
4. Ask the user only if the public interface or behavior priority genuinely changes the path; otherwise choose the smallest safe default.

Good question when truly blocking:

```text
Which public interface should own this behavior: CLI flag, API endpoint, or internal library fn? Recommended: API endpoint because current callers already route there.
```

For interface shape or test-seam choices, read
[deep-module-design](../refactor/references/deep-module-design.md). Record what
the selected seam catches and misses. Existing behavior/scope authorization is
enough for routine seam selection; ask only for a real unresolved choice.

Expected values come from an independent source: a worked example, known-good
literal, specification or separate oracle. Recomputing the expected value with
the implementation's own formula is tautological. Prove the test detects the
real missing/wrong behavior; a failure created only by a broken import is not RED.

## Anti-Pattern: Horizontal TDD

Do **not** write all tests first, then all implementation.

```text
WRONG:
  RED: test1, test2, test3, test4
  GREEN: impl1, impl2, impl3, impl4

RIGHT:
  RED->GREEN: test1->impl1
  RED->GREEN: test2->impl2
  RED->GREEN: test3->impl3
```

Bulk tests often encode imagined shapes instead of discovered behavior. Use vertical tracer bullets: one test proves one end-to-end slice, then the next test responds to what you learned.

## Cycle

### 1. RED — Write One Behavior Test

The test must:

- use the public interface only;
- describe what the user/caller observes;
- cover one logical behavior;
- fail for the expected missing behavior, not because of a typo/import problem.

Good:

```ts
test("user can checkout with a valid cart", async () => {
  const cart = createCart();
  cart.add(product);

  const result = await checkout(cart, paymentMethod);

  expect(result.status).toBe("confirmed");
});
```

Bad:

```ts
test("checkout calls paymentService.process", async () => {
  const mockPayment = jest.mock(paymentService);
  await checkout(cart, payment);
  expect(mockPayment.process).toHaveBeenCalledWith(cart.total);
});
```

Why bad: it tests internal wiring, not the checkout capability.

### 2. Verify RED

Run the focused test and confirm the failure is meaningful.

```bash
pytest tests/test_feature.py::test_specific_behavior -v
npm test -- tests/feature.test.ts -t "specific behavior"
```

For a new/missing behavior, an immediate pass means RED has not been demonstrated:
check whether the behavior already exists or the test misses the fault before
changing the test. Do not manufacture failure or duplicate an existing feature.
Characterization tests for behavior-preserving refactors should pass before the
change; they follow Refactor Test Discipline below instead.

If it errors because setup is wrong, fix setup until it fails for the right reason.

### 3. GREEN — Minimal Code

Write only enough code to pass the current test. No speculative abstractions, no unrelated cleanup, no hidden extra features.

Hardcoding is acceptable during GREEN if it helps prove the slice; clean it in REFACTOR after the next behavior demands generality.

### 4. REFACTOR — While Green Only

After the focused test passes:

- remove duplication;
- improve names;
- deepen modules: small interface, richer implementation hidden behind it;
- move logic to the owner of the data;
- simplify setup without changing behavior.

Run tests after each small refactor. If tests fail, undo and take a smaller step.

## Mocking Rule

Mock at system boundaries only:

- external APIs;
- payment/email providers;
- time/randomness;
- filesystem/network when real integration would be slow/flaky;
- databases only when no test DB or transaction rollback is practical.

Do not mock your own modules/classes just to make setup easier. If you must mock many internals, the interface is probably too coupled.

Design for testability:

- accept dependencies instead of constructing them deep inside logic;
- return values/results instead of mutating hidden state;
- prefer specific SDK-style boundary functions over one generic conditional fetcher;
- keep public surface small.

## Refactor Test Discipline

Refactors preserve behavior. If existing tests are weak or absent:

1. Add characterization tests through the public interface first.
2. Watch them fail only if they describe missing/uncaptured expected behavior; for preservation tests, watch them pass before refactor, then keep them passing.
3. Refactor in small steps.
4. Run full relevant suite before claiming done.

## Subagent / Plan Handoff

When delegating implementation, include explicit TDD evidence requirements:

```text
Use TDD. Return:
- RED command + expected failure output summary
- GREEN command + pass output summary
- refactor/test command summary
- changed files
```

For changed/missing behavior require observed RED and GREEN; “tests added” alone
is insufficient. For behavior-preserving characterization require baseline GREEN
and GREEN after the refactor. This section applies only when delegation is
already authorized; it does not initiate subagents.

## Output / Done Contract

For TDD work, final report includes:

- `RED`: focused command and failure reason observed;
- `GREEN`: focused command and pass result;
- `Regression`: wider suite/build result;
- `Changed`: files touched;
- `Residual risk`: untested boundaries or skipped cases.

## Common Pitfalls

1. **Writing tests after implementation.** Passing immediately proves little.
2. **Horizontal slicing.** Too many tests before one working tracer bullet.
3. **Testing private methods.** Public behavior should anchor tests.
4. **Mocking internals.** Usually hides coupling instead of testing behavior.
5. **Skipping RED because “obvious.”** Obvious code still breaks.
6. **Refactoring while RED.** Get green first.
7. **No proof in final report.** TDD without RED/GREEN evidence is a claim, not proof.

## Verification Checklist

- [ ] One behavior per cycle.
- [ ] Focused test failed for the expected reason before implementation.
- [ ] Minimal production code made it pass.
- [ ] Tests use public interface / observable behavior.
- [ ] Mocks limited to true system boundaries.
- [ ] Refactor done only while green.
- [ ] Focused and relevant broader checks reported.
