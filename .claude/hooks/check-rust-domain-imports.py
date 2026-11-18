#!/usr/bin/env python3
# Vendored from payment (.claude/hooks/check-php-domain-imports.py) on 2026-11-18. Adapted to this project: Rust use
# trees and inline paths for PHP use lines, the Makefile DOMAIN_CRATES allowlist for the own-vendor prefix, and no
# Commons/ wrapper exemption, because `check-domain-dependencies` admits none.
"""Domain imports check for the hexagon's domain layer.

The domain names only std and core, the crates of the DOMAIN_CRATES variable of the
Makefile (the allowlist `make check-domain-dependencies` enforces), and the domain itself:
`crate::application::domain`, `self::`, a child module, and `super::` while the chain stays
inside the domain. Inline paths count like use declarations, as they do for the Makefile
check.
"""

import re
import sys
from pathlib import Path
from typing import Final

from hooklib import rust
from hooklib.core import Violation, read_unit, run_check

# Domain layer, where the strict import restriction applies.
DOMAIN_PATTERN: Final = re.compile(r"(^|/)src/application/domain(/.+)?\.rs$")
DOMAIN_MODULE: Final = ("application", "domain")


def escaping(reference: rust.Reference, allowed: frozenset[str]) -> str:
    """The forbidden root a reference reaches, empty when the domain may name it.

    A use leaf whose head is no module of this crate names another crate and must be allowed.
    Inline paths reach here only when crate-, self-, or super-relative, because any other head
    (`Self::`, `Amount::new`) is a type in scope, so a dependency named inline is found apart."""
    if reference.escapes:
        return reference.written
    if reference.own is not None:
        return "" if reference.own[:2] == DOMAIN_MODULE else "crate::" + "::".join(reference.own)
    if reference.external in allowed:
        return ""
    return reference.written


def domain_import_violations(
    path: str,
    source: rust.Source,
    module: tuple[str, ...],
    allowed: frozenset[str],
    dependencies: frozenset[str],
) -> tuple[Violation, ...]:
    """Only std, core, the allowed crates, and the domain itself may be named here."""
    if not DOMAIN_PATTERN.search(path):
        return ()
    violations = []
    names = ", ".join(sorted(allowed - rust.STANDARD_CRATES))
    for reference in rust.references(source, module):
        reached = escaping(reference, allowed)

        if not reached:
            continue

        violations.append(Violation(
            line=source.line_of(reference.index),
            path=path,
            message=(
                f"domain names `{reached}`, only std, core, the crates {names}, and "
                f"crate::application::domain are allowed here"
            ),
        ))
    violations += inline_dependency_violations(path, source, allowed, dependencies)
    return tuple(sorted(violations, key=lambda violation: violation.line))


def inline_dependency_violations(
    path: str,
    source: rust.Source,
    allowed: frozenset[str],
    dependencies: frozenset[str],
) -> list[Violation]:
    """A Cargo dependency outside the allowlist named inline (`serde::Serialize` in a derive),
    outside any use declaration, which the use-tree scan never sees."""
    forbidden = sorted(dependencies - allowed)

    if not forbidden:
        return []
    pattern = re.compile(r"(?<![\w:])(" + "|".join(map(re.escape, forbidden)) + r")\s*::")
    covered = [(match.start(), match.end()) for match in rust.USE_DECLARATION.finditer(source.clean)]
    names = ", ".join(sorted(allowed - rust.STANDARD_CRATES))
    return [
        Violation(
            line=source.line_of(match.start()),
            path=path,
            message=(
                f"domain names `{match.group(1)}::` inline, only std, core, the crates {names}, "
                f"and crate::application::domain are allowed here"
            ),
        )
        for match in pattern.finditer(source.clean)
        if not any(start <= match.start() < end for start, end in covered)
    ]


def in_scope(path: Path) -> bool:
    """Whether the path is a Rust source under src/ or tests/ with a Rust context above it."""
    return (
        bool(rust.SRC_TESTS_PATTERN.search(path.as_posix()))
        and path.is_file()
        and rust.context_present(path)
    )


def file_violations(path: Path) -> tuple[Violation, ...]:
    """The domain-import violations for one file, against the allowlist of its own Makefile.

    Reading the allowlist per file (never from a value written here) is what keeps the hook and
    `make review` on one list, and lets the check travel to any project that declares it."""
    if not DOMAIN_PATTERN.search(path.as_posix()):
        return ()
    allowed = rust.domain_crates(path)
    module = rust.module_path(path)

    if allowed is None or module is None:
        print(
            f"{path.as_posix()}: cannot resolve the domain crates from the Makefile "
            f"(DOMAIN_CRATES) or the module path under src/, domain-imports not checked",
            file=sys.stderr,
        )
        return ()
    unit = read_unit(path)
    return domain_import_violations(
        unit.path,
        rust.Source.from_rust(unit.text),
        module,
        allowed | rust.STANDARD_CRATES,
        rust.dependency_crates(path),
    )


def main() -> int:
    return run_check(in_scope, file_violations)


if __name__ == "__main__":
    sys.exit(main())
