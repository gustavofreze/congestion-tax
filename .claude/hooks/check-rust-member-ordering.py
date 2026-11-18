#!/usr/bin/env python3
# Vendored from payment (.claude/hooks/check-php-member-ordering.py) on 2026-11-18. Adapted to this project: Rust fn
# signatures for PHP functions, two tiers because a trailing impl IntoIterator parameter plays the variadic role, no
# call-site check because Rust has no named argument, the start and end pairs matched by snake_case token instead of
# enumerated, and trait impls, axum handlers, and rstest cases exempt as the contract-fixed signatures.
"""Member ordering check for Rust declarations.

Parameters of a constructor, an associated function, or a method order by identifier
length ascending, alphabetical tie-break, with semantic pairs kept adjacent in natural
order. A start and its end are always a pair, start first: any two names that differ only
by a `start`/`end` or `starts`/`ends` token (`starts_on` and `ends_on`, `window_start` and
`window_end`). The receiver (`self`, `&self`, `&mut self`) is not a parameter here.

PHP orders within required, defaulted, and variadic tiers. Rust has no defaulted parameter,
and an `impl IntoIterator` parameter plays the variadic role (rust-code-style § Parameter
ordering), so two tiers remain: the other parameters, then the `impl IntoIterator` ones, each
ordered by the same criterion. PHP also orders named arguments at call sites.
Rust calls are positional, so a call site follows its declaration and is not checked.

Exempt, because a contract outside the signature fixes the order: a method inside a trait
impl block (`impl Trait for Type`, the trait declares the order, and the trait declaration
itself is checked when it is ours), an axum handler (a parameter typed by an axum extractor,
the body extractor goes last), a signature with a destructuring pattern (it binds by the
shape of its type, not by a name), and an rstest test (its parameters follow its `#[case]`
values, as a PHP data provider feeds its test method).
"""

import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from hooklib import rust
from hooklib.core import Violation, read_unit, run_check

# Semantic pairs other than a start and its end: natural order wins between the two members.
SEMANTIC_PAIRS: Final = (
    ("from", "to"),
    ("first", "last"),
    ("created_at", "updated_at"),
    ("before", "after"),
    ("min", "max"),
    ("code", "body"),
    ("params", "types"),
    ("accumulator", "initial"),
)

# Each member maps to (first, second, position), the pair sorts as a unit at the lead key.
PAIR_MEMBER: Final = {
    member: (first, second, position)
    for first, second in SEMANTIC_PAIRS
    for position, member in enumerate((first, second))
}

# The tokens that make a start and its end, each mapped to its partner and its position.
START_END_TOKENS: Final = {"start": ("end", 0), "starts": ("ends", 0), "end": ("start", 1), "ends": ("starts", 1)}

FUNCTION_DECLARATION: Final = re.compile(r"\bfn\s+(\w+)\s*")
TRAIT_IMPL_HEADER: Final = re.compile(r"\bimpl\b[^{;]*?\bfor\b[^{;]*\{")
RSTEST_ATTRIBUTE: Final = re.compile(r"#\[\s*(?:rstest::)?rstest\b")
ATTRIBUTE: Final = re.compile(r"#\[[^\]]*\]")
RECEIVER: Final = re.compile(r"^&?\s*(?:'\w+\s+)?(?:mut\s+)?self\b")
IDENTIFIER_PATTERN: Final = re.compile(r"^(?:ref\s+)?(?:mut\s+)?(\w+)$")
VARIADIC_TYPE: Final = re.compile(r"^impl\s+IntoIterator\b")

# axum extractors, whose order axum fixes: the request parts first, the one body extractor last.
AXUM_EXTRACTOR: Final = re.compile(
    r"\b(?:State|Json|Path|Query|Form|Extension|ConnectInfo|HeaderMap|Request|Bytes|RawQuery|OriginalUri)\b"
)


# --- Ordering -------------------------------------------------------------------


@dataclass(frozen=True)
class Ordering:
    """Sole owner of the parameter-ordering logic."""

    pair_member: dict[str, tuple[str, str, int]]

    def sorted(self, names: list[str]) -> list[str]:
        """The names in the required order."""
        present = set(names)
        return sorted(names, key=lambda name: self.key(name, present))

    def key(self, name: str, present: set[str]) -> tuple[int, str, int]:
        """Length, then alphabet, with both members of a present pair at the lead key."""
        entry = self.pair_member.get(name) or start_end_pair(name)
        if entry is not None:
            first, second, position = entry
            partner = second if position == 0 else first
            if partner in present:
                return (len(first), first, position)
        return (len(name), name, 0)


def start_end_pair(name: str) -> tuple[str, str, int] | None:
    """The (start, end, position) a name takes in a start and end pair, or None when no token of
    the name is a start or an end."""
    tokens = name.split("_")
    for index, token in enumerate(tokens):
        if token not in START_END_TOKENS:
            continue
        partner_token, position = START_END_TOKENS[token]
        partner = "_".join([*tokens[:index], partner_token, *tokens[index + 1:]])
        return (name, partner, 0) if position == 0 else (partner, name, 1)
    return None


PARAMETER_ORDERING: Final = Ordering(pair_member=PAIR_MEMBER)


@dataclass(frozen=True)
class ParameterList:
    """One declaration's parameters in source order."""

    line: int
    owner: str
    names: list[str]
    variadic: frozenset[str] = frozenset()

    def required(self) -> list[str]:
        fixed = [name for name in self.names if name not in self.variadic]
        trailing = [name for name in self.names if name in self.variadic]
        return PARAMETER_ORDERING.sorted(fixed) + PARAMETER_ORDERING.sorted(trailing)

    def out_of_order(self) -> bool:
        return len(self.names) >= 2 and self.names != self.required()


# --- Structure ----------------------------------------------------------------


def angle_aware_pieces(text: str) -> list[str]:
    """The comma-separated pieces of a parameter list, ignoring separators nested in brackets
    or generics. A `>` closing `->` is an arrow, never a generic."""
    pieces, depth, current = [], 0, []
    for position, character in enumerate(text):
        if character in "([{<":
            depth += 1

        if character in ")]}" or (character == ">" and text[position - 1:position] != "-"):
            depth -= 1

        if character == "," and depth == 0:
            pieces.append("".join(current))
            current = []
            continue
        current.append(character)

    tail = "".join(current).strip()

    if tail:
        pieces.append(tail)

    return pieces


def generics_end(text: str, start: int) -> int:
    """The position just past the generic parameter list opening at start, or start when none."""
    if text[start:start + 1] != "<":
        return start
    depth = 0
    for position in range(start, len(text)):
        if text[position] == "<":
            depth += 1
        if text[position] == ">" and text[position - 1] != "-":
            depth -= 1
            if depth == 0:
                return position + 1
    return len(text)


def pattern_and_type(piece: str) -> tuple[str, str]:
    """The pattern and the type of one parameter, split at the first lone `:`."""
    match = re.search(r"(?<!:):(?!:)", piece)
    if match is None:
        return piece.strip(), ""
    return piece[:match.start()].strip(), piece[match.end():].strip()


# --- Extraction ---------------------------------------------------------------


def exempt_ranges(source: rust.Source) -> list[tuple[int, int]]:
    """The bodies of every trait impl block, whose method signatures the trait fixes."""
    spans = rust.bracket_spans(source.clean)
    return [
        (header.end() - 1, spans.get(header.end() - 1, len(source.clean)))
        for header in TRAIT_IMPL_HEADER.finditer(source.clean)
    ]


def rstest_lines(source: rust.Source) -> set[int]:
    """Declaration lines of rstest tests, whose parameters follow their `#[case]` values."""
    lines = set()
    for attribute in RSTEST_ATTRIBUTE.finditer(source.clean):
        function = FUNCTION_DECLARATION.search(source.clean, attribute.end())
        if function:
            lines.add(source.line_of(function.start()))
    return lines


def declared_signatures(source: rust.Source) -> list[ParameterList]:
    """Every fn declaration outside a trait impl, with its named parameters, skipping the
    signatures a destructuring pattern or an axum extractor fixes."""
    clean = source.clean
    spans = rust.bracket_spans(clean)
    exempt = exempt_ranges(source)
    signatures = []

    for match in FUNCTION_DECLARATION.finditer(clean):
        if any(start < match.start() < end for start, end in exempt):
            continue
        open_paren = generics_end(clean, match.end())
        if clean[open_paren:open_paren + 1] != "(":
            continue
        inner = clean[open_paren + 1: spans.get(open_paren, len(clean))]
        pieces = [ATTRIBUTE.sub("", piece).strip() for piece in angle_aware_pieces(inner)]
        parameters = [pattern_and_type(piece) for piece in pieces if piece and not RECEIVER.match(piece)]

        if any(AXUM_EXTRACTOR.search(kind) for _, kind in parameters):
            continue
        names = [IDENTIFIER_PATTERN.match(pattern) for pattern, _ in parameters]

        if not all(names):
            continue
        signatures.append(ParameterList(
            line=source.line_of(match.start()),
            owner=match.group(1),
            names=[name.group(1) for name in names],
            variadic=frozenset(
                name.group(1) for name, (_, kind) in zip(names, parameters) if VARIADIC_TYPE.match(kind)
            ),
        ))
    return signatures


# --- Checks -------------------------------------------------------------------


def parameter_violations(path: str, source: rust.Source) -> tuple[Violation, ...]:
    """Parameter ordering on every declaration, skipping the rstest tests whose parameters
    follow their cases."""
    exempt = rstest_lines(source)
    return tuple(
        Violation(
            line=signature.line,
            path=path,
            message=(
                f"parameter order in `{signature.owner}()` is "
                f"({', '.join(signature.names)}), "
                f"required ({', '.join(signature.required())})"
            ),
        )

        for signature in declared_signatures(source)
        if signature.line not in exempt
        and signature.out_of_order()
    )


# --- Shell --------------------------------------------------------------------


def in_scope(path: Path) -> bool:
    """Whether the path is a Rust source under src/ or tests/ with a Rust context above it."""
    return (
        bool(rust.SRC_TESTS_PATTERN.search(path.as_posix()))
        and path.is_file()
        and rust.context_present(path)
    )


def file_violations(path: Path) -> tuple[Violation, ...]:
    """The ordering violations for one file: declared parameters."""
    unit = read_unit(path)
    return parameter_violations(unit.path, rust.Source.from_rust(unit.text))


def main() -> int:
    return run_check(in_scope, file_violations)


if __name__ == "__main__":
    sys.exit(main())
