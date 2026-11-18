---
name: rust-reviewing-conformance
description: Read-only audit of code against the rules, AGENTS.md, and the contract, producing a Markdown findings report. Use when asked to audit code, review conformance, or check rule adherence.
---
<!-- Vendored from payment (.claude/skills/php-reviewing-conformance/SKILL.md) on 2026-11-18. Adapted to this project: CLAUDE.md became AGENTS.md (CLAUDE.md only imports it), the rule set is named file by file with the target's rust-* and shared rule names, the spec read became a read of openapi.yaml and the docs pages because no spec skill exists and <spec-root> is absent, and the fix routes name the rust-* skills. -->

# Review conformance

Read-only audit against the rules, the `AGENTS.md`, and the contract. Produces findings in Markdown, never edits.

Target: the path to audit, or the whole service when none is named.

## When to use

- Audit code, review conformance, check adherence to the rules, produce a findings report.
- Confirm a change holds against `AGENTS.md`, the layer rules, and the contract before it ships.

## When NOT to use

- Fix what it finds: this action only reports. The fix is the layer action responsibility (`rust-creating-driver`,
  `rust-creating-driven`, `rust-creating-query`, `rust-creating-application`, or `rust-implementing-features`).
- Decide whether a design pattern belongs: that is a selection call owned by `rust-applying-design-patterns`, not a
  conformance audit.
- Stand in for reading the contract on domain work: the business behavior is anchored on the spec at `<spec-root>` when
  set, and otherwise on `openapi.yaml`, the docs pages under `docs/`, and `AGENTS.md`. This skill consults them but does
  not replace that read.

## Rules applied

- Every rule under `.claude/rules/`, each file checked against the rules whose `paths:` frontmatter covers it (a rule
  with no `paths:` applies to every file):
  - Architecture and style: `rust-architecture`, `rust-code-style`, `rust-design-principles`, `rust-skeleton-tooling`.
  - Application: `rust-hex-application-domain`, `rust-hex-application-handlers`.
  - Adapters and the read side: `rust-hex-driven`, `rust-hex-driven-repository`, `rust-hex-driver-http`,
    `rust-hex-query`, `rust-hex-sql-queries`.
  - Tests: `rust-testing`, `rust-testing-unit`, `rust-testing-integration`.
  - Repository files: `database-migration`, `docker-images`, `github-workflows`, `shell-scripts`, `skeleton-layout`,
    `web-documentation`.
- `AGENTS.md`, both halves (`CLAUDE.md` only imports it): the project's declarations above the dividing line, and the
  toolkit contract below it (the reading table and token resolution, capability absence, commands, architecture,
  dependency policy, global defaults, conventions). A rule stated in `AGENTS.md` is audited exactly as one stated in a
  rule file.
- The contract (`openapi.yaml` and the docs pages, or the spec at `<spec-root>` when set), when business rule is in
  play.

## Assembly order

1. Define the scope of paths to audit.
2. Anchor on the contract if there is business behavior.
3. Check each file against the rules that cover it, and the `AGENTS.md`.
4. Write the findings in structured Markdown: file, line, violated rule, evidence.

## Completeness gate

- [ ] Read-only: no code edit.
- [ ] Each finding cites the rule and the location (file and line).
- [ ] Conformance findings separated from style preference.
- [ ] Report in structured Markdown.

## Additional resources

- **`assets/findings-report-template.md`**: Template for the structured Markdown findings report. Copy and fill in:
  scope, date, and one entry per finding with file, line, rule, evidence, and finding.

## Does not do

- Does not invent a violation: no evidence, no finding.
