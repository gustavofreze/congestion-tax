<!-- Vendored from payment (.claude/skills/php-writing-tests/references/coverage-and-mutation.md) on 2026-11-18. Adapted to this project: the PHPUnit coverage text and the Infection logs became the cargo-llvm-cov gate and HTML report and the cargo-mutants outcome files under reports/, and the incremental tests versus tests-full split was dropped because make tests mutates the whole source. -->

# Coverage and mutation read-order procedure

Read this when verifying that a suite reaches 100% coverage and a fully caught mutation run. None of it is a per-file
invariant, so it is NOT path-injected. The invariants (100% line coverage, every mutant caught, refactor rather than
work around the tool) live in `rust-testing` (§ Coverage and mutation). This file holds the report-reading procedure
that used to sit inside that rule.

## Determining coverage and mutation status

To determine whether coverage is 100% and all mutants are caught, read the gate and the report files directly. Do NOT
guess, infer from the test output, or claim success without reading them.

- **Coverage gate**: `make tests` runs `cargo llvm-cov report --summary-only --fail-under-lines 100`, so the target
  fails below 100 percent of lines and never reaches the mutation step. A failing `make tests` before `cargo mutants`
  runs is a coverage or test failure, never a mutation one.
- **Coverage report**: `reports/coverage/html/index.html`. Per-file and per-line coverage. Use it to locate the lines a
  test must reach.
- **Mutation outcome**: `reports/mutants.out/`. A successful run requires `missed.txt` and `timeout.txt` to be empty.
  `caught.txt` lists the mutants a test caught and `unviable.txt` the ones that did not compile, and neither counts
  against the run. Any line in `missed.txt` or `timeout.txt` means the task is incomplete.
- **Mutation details**: `reports/mutants.out/outcomes.json`, the per-mutant record (file, line, replacement, outcome),
  and `reports/mutants.out/debug.log`. Use them to locate the specific test that must be added or strengthened.

When reviewing testing work, always read `reports/mutants.out/missed.txt` and `reports/mutants.out/timeout.txt` after a
`make tests` that reached the mutation step, before declaring the task complete. The suite runs only through
`make tests` (every Cargo command runs inside the container image), and `reports/` is gitignored, so on a fresh clone
the files do not exist until `make tests` has run. If `reports/` is missing or stale, run `make tests` first.
`reports/mutants.out.old/` is the previous run, which cargo-mutants keeps aside. Never read it as the current result.

`make tests` measures coverage over the whole suite and mutates the whole source outside the composition root (the
`exclude_globs` of `.cargo/mutants.toml`), so its mutation outcome speaks for every file. `minimum_test_timeout` there
keeps a slow run against PostgreSQL from reporting a caught mutant as a timeout.

## Read order (keep context small)

The command output of `make tests` is not a source of truth for the mutation status and must never be used to determine
it. Read the files in this order, pulling the minimum into context:

1. **`reports/mutants.out/missed.txt` and `timeout.txt` first.** Smallest files, and the gate. Apply the pass criterion
   already defined above (both empty).
2. **`reports/mutants.out/outcomes.json` only when the gate is not green.** The large per-mutant file. Open it solely
   to locate the file, line, and replacement behind a missed or timed-out mutant, filtered to those outcomes.
3. **`reports/coverage/html/` filtered.** Open the page of the file below 100% only. Never read the whole report into
   context when the file named by the failed gate is enough.
