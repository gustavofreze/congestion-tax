#!/usr/bin/env python3
# Vendored from payment (.claude/hooks/check-php-hexagonal-layers.py) on 2026-11-18. Adapted to this project: layers
# are read from the module path of the file (crate::application::domain, crate::driver, and so on) instead of PSR-4
# namespaces, use trees and inline crate-, self-, and super-relative paths are both checked, and two rules were added
# that the PHP hook leaves unstated: driven never names driver, and the application never names a root module.
"""Hexagonal layer check for the service hexagon under src/.

Dependencies point inward only, and the CQRS read side stays segregated from the write
side. Root modules of the crate (src/lib.rs, src/main.rs, src/routes.rs, and the settings
modules the binary declares) are the composition root that wires every layer, and are out
of scope. Tests cross layers freely and are out of scope.
"""

import sys
from pathlib import Path
from typing import Final

from hooklib import rust
from hooklib.core import Violation, read_unit, run_check

# Layer detection by module path. Domain is matched before application because it nests inside it.
LAYER_MODULES: Final = (
    ("domain", ("application", "domain")),
    ("application", ("application",)),
    ("driver", ("driver",)),
    ("driven", ("driven",)),
    ("query", ("query",)),
)


def layer_of(module: tuple[str, ...]) -> str:
    """The hexagon layer of a module path, or an empty string for a root or out-of-layer module."""
    for name, prefix in LAYER_MODULES:
        if module[:len(prefix)] == prefix:
            return name
    return ""


def forbidden(layer: str, imported: tuple[str, ...]) -> str:
    """The reason a path of this crate is forbidden from this layer, empty when allowed."""
    named = "crate::" + "::".join(imported)
    target = layer_of(imported)

    if layer == "domain" and target != "domain":
        return (
            f"domain names `{named}`, the domain depends on no other layer, "
            f"only crate::application::domain::* is allowed here"
        )
    if layer == "application" and target not in ("domain", "application"):
        return (
            f"application names `{named}`, dependencies point inward, the application "
            f"never reaches out to driver, driven, the query read side, or the composition root"
        )
    if layer == "driver" and target == "driven":
        return (
            f"driver names `{named}` from the driven layer, a driver adapter depends only on "
            f"application ports and application or domain errors, never on a driven type"
        )
    if layer == "driver" and target == "query":
        return (
            f"driver names `{named}` from the query read side, the write and read sides "
            f"are segregated (CQRS)"
        )
    if layer == "driven" and target == "driver":
        return (
            f"driven names `{named}` from the driver layer, a driven adapter implements an "
            f"outbound port and depends only on application ports and domain types"
        )
    if layer == "driven" and target == "query":
        return (
            f"driven names `{named}` from the query read side, the write and read sides "
            f"are segregated (CQRS)"
        )
    if layer == "query" and target in ("domain", "application", "driver", "driven"):
        return (
            f"query names `{named}` from the write side, each read slice is self-contained "
            f"and never reuses application, driver, or driven code (CQRS)"
        )
    return ""


def hexagonal_violations(path: str, source: rust.Source, module: tuple[str, ...]) -> tuple[Violation, ...]:
    """The layer-coupling violations for one file, considering paths of this crate only."""
    layer = layer_of(module)

    if not layer:
        return ()
    violations = []
    for reference in rust.references(source, module):
        if reference.own is None:
            continue

        message = (
            f"`{reference.written}` climbs above the crate root"
            if reference.escapes
            else forbidden(layer=layer, imported=reference.own)
        )

        if not message:
            continue

        violations.append(Violation(
            line=source.line_of(reference.index),
            path=path,
            message=message,
        ))
    return tuple(violations)


def in_scope(path: Path) -> bool:
    """Whether the path is a Rust source under src/ with a Rust context above it."""
    return (
        bool(rust.SRC_PATTERN.search(path.as_posix()))
        and path.is_file()
        and rust.context_present(path)
    )


def file_violations(path: Path) -> tuple[Violation, ...]:
    """The hexagonal violations for one file."""
    module = rust.module_path(path)

    if module is None:
        print(
            f"{path.as_posix()}: cannot resolve the module path under the src/ of the nearest "
            f"Cargo.toml, hexagonal conformance not checked",
            file=sys.stderr,
        )
        return ()
    unit = read_unit(path)
    return hexagonal_violations(unit.path, rust.Source.from_rust(unit.text), module)


def main() -> int:
    return run_check(in_scope, file_violations)


if __name__ == "__main__":
    sys.exit(main())
