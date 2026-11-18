# Vendored from payment (.claude/hooks/hooklib/__init__.py) on 2026-11-18. Adapted to this project: the language
# module beside core is rust, not php.
"""Shared internal library for the Claude Code hooks in this folder.

Self-contained: Python stdlib only, resolved from this directory because every entry
point lives beside it. `core` holds the language-agnostic scaffolding (payload parsing,
violations, reporting). `rust` is the Rust language module behind the language seam, a
later language module can sit beside it without touching `core`.
"""
