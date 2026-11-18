<!-- Vendored from payment (.claude/skills/php-reviewing-conformance/assets/findings-report-template.md) on 2026-11-18. Adapted to this project: CLAUDE.md became AGENTS.md, the spec skill became a read of openapi.yaml and the docs pages, and the fix owners name the rust-creating-* and rust-writing-* skills. -->

# Conformance findings: <scope>

Audit date: <date> Scope: <path or "whole service"> Rules applied: all `.claude/rules/`, `AGENTS.md`, and the contract
(`openapi.yaml` and the docs pages) where business behavior is in scope.

---

## Summary

| Metric               | Count |
|----------------------|-------|
| Conformance findings | 0     |

---

## Findings

### <file-path>:<line>

**Rule:** `<rule-file>` section `<section-heading>` **Evidence:** `<quoted code or prose that violates the rule>`
**Finding:** <one sentence stating what is wrong and why it violates the rule>

---

## No findings

If no violations were found, replace the Findings section with:

> No conformance violations found in the audited scope. All files satisfy the rules and `AGENTS.md`.

---

## Notes

- Conformance findings only. Style preferences not listed here unless a rule mandates them.
- Each finding cites the rule and location. No finding without evidence.
- This report is read-only. Fixes are the responsibility of the matching layer skill (`rust-creating-*`,
  `rust-writing-*`).
