# Feature Ledger

:PROPERTIES:
:ID: 59fd6724c1aa5324f019b10a55be2bd4e14c63e1
:TYPE: FEATURE
:STATUS: DRAFT
:END:

Feature ledger for the `xiuxian-testing` library crate. Track user-facing or system-facing capabilities implemented in this package.

## Landed Features

1. Contract testing kernel (`contracts::*`) for deterministic rule-pack based audits.
2. Scenario framework (`scenario::*`) for snapshot-driven fixture verification.
3. Test policy and structure validation (`policy::*`, `validation::*`) including:
   - workspace rule overrides,
   - deterministic post-harness leaf-file bloat detection,
   - advisory repeated-namespace path warnings for crate `src/` and `tests/`
     source trees.
4. Performance gate kernel (`performance::*`, feature-gated) with:
   - budget model (`PerfBudget`),
   - run configuration (`PerfRunConfig`),
   - quantile report (`PerfReport`),
   - sync/async budget runners,
   - unified budget assertion and JSON reporting contract.
