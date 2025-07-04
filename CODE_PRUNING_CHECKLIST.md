# Code-Pruning & Zero-Warning Clean-up Plan

This document enumerates **all** items that must be addressed in order to bring the Bingo workspace to the following standard:

* Builds without **any** compiler or Clippy warnings (`-D warnings`) on **all** targets.
* Contains **no** dead/unused/placeholder/legacy/future code.
* Contains **no** `TODO`, `FIXME`, `#[allow(dead_code)]`, `#[allow(unused_…)]`, or similar suppression markers.
* Passes the full test-suite in `debug` **and** `release` mode.

Every finding below is written as an actionable task that can be turned into an issue or a PR.  Please keep the **ID** stable – it will be used for cross-referencing.

---

## 1. Build/Workspace level

| ID | Task | Location | Suggested fix | Status |
|----|------|----------|---------------|--------|
| **B-1** | Remove unused Cargo-manifest keys that trigger warnings: `package.lints`, `package.rust-2021-lints`. | `crates/bingo-api/Cargo.toml` | Delete the keys or move them under `[package.metadata]` / `[workspace.lints]` as appropriate. | ✅ Done |
| **B-2** | Activate `#![deny(warnings)]` for every crate. | All crate roots (`lib.rs`, `main.rs`) | Add `#![deny(warnings, missing_docs, clippy::all, clippy::pedantic)]` after temporarily fixing all outstanding warnings. | ✅ Done |
| **B-3** | Run `cargo clippy --workspace --all-targets -- -D warnings` in CI. | `.github/workflows/ci.yml` (or equivalent) | Add a Clippy step that fails the build on warnings. | ✅ Done |

## 2. Compiler errors (must-fix blockers)

| ID | Task | Location | Suggested fix | Status |
|----|------|----------|---------------|--------|
| **E-1** | Rename shadowed variable so that `stats` is in scope.  Current build fails with `cannot find value stats in this scope`. | `crates/bingo-core/tests/serialization_performance_test.rs`, lines 61-105 | Replace `let _stats = …` with `let stats = …` and update usages. | ✅ Done |

## 3. Dead-code / allow-attribute clean-up

Remove the suppression attributes **and** either delete the unused code or make it used.  A non-exhaustive search produced the list below – rerun `rg -n "allow([^)]*dead_code"` after each clean-up round.

| ID | Task | File & line | Status |
|----|------|-------------|--------|
| **D-1** | Eliminate `#[allow(dead_code)]` on helper inside Incremental Processor. | `crates/bingo-api/src/incremental_processor.rs:23` | ✅ Done |
| **D-2** | Remove six dead-code allows in JSON test runner helpers. | `crates/bingo-api/tests/json_test_runner.rs:30,45,58,68,80` | ✅ Done |
| **D-3** | Remove dead-code allow on `RuleNodeKind::Debug` (or delete the variant). | `crates/bingo-core/src/rete_network.rs:362` | ✅ Done |
| **D-4** | Drop crate-wide lax policy `#![allow(missing_docs, unused_imports, unused_variables, dead_code)]`. | `crates/bingo-calculator/src/lib.rs:1` | ✅ Done |

Additional `unused_imports` / `unused_variables` allows:

| ID | File | Status |
|----|------|--------|
| **D-5** | `crates/bingo-core/tests/advanced_aggregation_integration_test.rs:1` | ✅ Done |
| **D-6** | `crates/bingo-core/tests/lazy_aggregation_integration_test.rs:6` | ✅ Done |
| **D-7** | `crates/bingo-core/tests/serialization_performance_test.rs:6` | ✅ Done |

## 4. Outstanding Clippy warnings (clean-up required)

The following warnings were observed during `cargo clippy`.  Fix each and then remove any remaining allow-rules.  Use `clippy --fix --allow-dirty --allow-staged` for the trivial ones.

| ID | Clippy lint | File | Hint | Status |
|----|-------------|------|------|--------|
| **C-1** | `clippy::collapsible_match` (4 instances) | `crates/bingo-api/src/error.rs` | Merged option pattern into outer `match`. | ✅ Done |
| **C-2** | `clippy::io_other_error` | `crates/bingo-api/src/error.rs:271` | Replaced with `std::io::Error::other`. | ✅ Done |
| **C-3** | `clippy::result_large_err` | `crates/bingo-api/src/error.rs:321` | Boxed large `Err` variant via new `ApiRejection` alias. | ✅ Done |
| **C-4** | `clippy::let_and_return` | `crates/bingo-api/src/incremental_processor.rs:415-416` | Returned expression directly. | ✅ Done |
| **C-5** | `clippy::explicit_auto_deref` | `crates/bingo-api/src/optimized_conversions.rs:301` | Simplified to `&ctx.borrow()`. | ✅ Done |

Re-run Clippy after these to surface secondary lints.

## 5. Disabled / ignored tests & benches

| ID | Task | File | Status |
|----|------|------|--------|
| **T-1** | Decide fate of `debugging_test.rs.disabled`. Delete or enable. | (file removed) | ✅ Deleted |
| **T-2** | Audit all `#[ignore]` tests (performance-heavy). | Multiple files | ✅ Reviewed |

## 6. Unsafe blocks audit

Five `unsafe` blocks remain in the workspace.

| ID | File & line | Purpose | Action | Status |
|----|-------------|---------|--------|--------|
| **U-1** | `crates/bingo-api/src/tracing_setup.rs:213` | Set env vars in test. | Removed unnecessary `unsafe` block. | ✅ Done |
| **U-2** | `crates/bingo-api/src/tracing_setup.rs:225` | Remove env vars in test. | Removed unnecessary `unsafe` block. | ✅ Done |
| **U-3** | `crates/bingo-api/tests/json_test_framework.rs:223` | Env var set. | Replaced `unsafe` with safe call. | ✅ Done |
| **U-4** | `crates/bingo-api/tests/json_test_framework.rs:229` | Env var set. | Replaced `unsafe` with safe call. | ✅ Done |
| **U-5** | `crates/bingo-core/src/memory.rs:63` | Windows API FFI call. | Added detailed SAFETY comment explaining correctness. | ✅ Done |

## 7. Documentation & TODO markers

Search confirmed that **source** code is free of `TODO:` markers, but several still live in `ANALYSIS.md`.  Either update or archive that document.  No action required in compiled crates.

## 8. Final verification checklist

1. `cargo clean && cargo test --workspace --all-features --release`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `rg -n "TODO|FIXME|allow(dead_code)|\.disabled" crates` → should yield **no** hits.
4. Run `cargo bench` to ensure perf tests still build.

Once all items are closed the workspace will be **warning-free, dead-code-free and fully clean**.
