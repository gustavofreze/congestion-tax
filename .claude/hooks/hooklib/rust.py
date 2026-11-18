# Vendored from payment (.claude/hooks/hooklib/php.py) on 2026-11-18. Adapted to this project: the PHP language
# module became a Rust one (Cargo.toml for composer.json, module paths for PSR-4 namespaces, use trees for use lines).
"""Rust language module: scope patterns, lexical blanking, use-tree expansion, module paths,
and the domain crate allowlist read from the Makefile.

This module is the language seam. Entry points take everything Rust-specific from here, so
another language module can sit beside it without touching hooklib.core. Callers gate on
context_present() to no-op silently outside a Rust repo.
"""

import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Iterator

from hooklib.core import project_root

# In-scope patterns for Rust sources.
SRC_TESTS_PATTERN: Final = re.compile(r"(^|/)(src|tests)/.+\.rs$")
SRC_PATTERN: Final = re.compile(r"(^|/)src/.+\.rs$")

# Literal openings: a raw string (`r"`, `r#"`, `br#"`), a prefixed string (`b"`, `c"`), and a whole
# character literal (`'a'`, `'\n'`, `b'x'`), which a lifetime (`'a`) never matches.
RAW_STRING_OPENING: Final = re.compile(r'[bc]?r(#*)"')
STRING_OPENING: Final = re.compile(r'[bc]"')
CHARACTER_LITERAL: Final = re.compile(r"b?'(?:\\(?:u\{[0-9A-Fa-f]+\}|x[0-9A-Fa-f]{2}|.)|[^\\'\n])'")

# Directories Cargo writes into: build output and the container Cargo home with the registry sources.
BUILD_DIRECTORIES: Final = frozenset({"target", ".cargo-home"})

# The crates every Rust file may name without a Cargo dependency.
STANDARD_CRATES: Final = frozenset({"std", "core"})

# One use declaration, public or not, up to its closing semicolon.
USE_DECLARATION: Final = re.compile(r"(?<![\w:])(?:pub(?:\s*\([^)]*\))?\s+)?use\s+([^;]+);")

# A path that starts at the crate root or relative to the current module, outside a use declaration.
INLINE_OWN_PATH: Final = re.compile(r"(?<![\w:])(?:crate|self|super)(?:\s*::\s*\w+)+")

# A child module declared in the file, by `mod name;` or an inline `mod name { ... }`.
MODULE_DECLARATION: Final = re.compile(r"(?<![\w:])mod\s+(\w+)\s*([;{])")

# The DOMAIN_CRATES variable of the Makefile, the allowlist behind `check-domain-dependencies`.
DOMAIN_CRATES_VARIABLE: Final = re.compile(r"^DOMAIN_CRATES\s*[:?]?=\s*(.+)$", re.MULTILINE)

# The [dependencies] table of Cargo.toml and one dependency key inside it.
DEPENDENCIES_TABLE: Final = re.compile(r"^\[dependencies\]\s*$(.*?)(?=^\[|\Z)", re.MULTILINE | re.DOTALL)
DEPENDENCY_KEY: Final = re.compile(r"^([A-Za-z0-9_-]+)\s*=", re.MULTILINE)


@dataclass(frozen=True)
class Span:
    """One lexical literal: a comment or a string, by kind and character range."""

    kind: str
    start: int
    end: int


def literal_spans(text: str) -> Iterator[Span]:
    """Every comment, string, and character literal in Rust source, in order.

    A scanner rather than one regular expression, because Rust block comments nest, raw strings
    close on a counted run of `#`, and a `'` opens either a character literal or a lifetime."""
    index, length = 0, len(text)
    while index < length:
        if text.startswith("//", index):
            end = text.find("\n", index)
            end = length if end == -1 else end
            yield Span(kind="comment", start=index, end=end)
            index = end
            continue
        if text.startswith("/*", index):
            end = block_comment_end(text, index)
            yield Span(kind="comment", start=index, end=end)
            index = end
            continue
        prefixed = not word_before(text, index)
        raw = RAW_STRING_OPENING.match(text, index) if prefixed else None
        if raw:
            closing = '"' + raw.group(1)
            end = text.find(closing, raw.end())
            end = length if end == -1 else end + len(closing)
            yield Span(kind="string", start=index, end=end)
            index = end
            continue
        quoted = STRING_OPENING.match(text, index) if prefixed else None
        if quoted or text[index] == '"':
            end = string_end(text, quoted.end() if quoted else index + 1)
            yield Span(kind="string", start=index, end=end)
            index = end
            continue
        character = CHARACTER_LITERAL.match(text, index) if prefixed or text[index] == "'" else None
        if character:
            yield Span(kind="string", start=index, end=character.end())
            index = character.end()
            continue
        index += 1


def block_comment_end(text: str, start: int) -> int:
    """The position just past the block comment opened at start, nested comments included."""
    depth, cursor = 1, start + 2
    while cursor < len(text) and depth:
        if text.startswith("/*", cursor):
            depth, cursor = depth + 1, cursor + 2
            continue
        if text.startswith("*/", cursor):
            depth, cursor = depth - 1, cursor + 2
            continue
        cursor += 1
    return cursor


def string_end(text: str, cursor: int) -> int:
    """The position just past the closing quote of a string whose body starts at cursor."""
    while cursor < len(text) and text[cursor] != '"':
        cursor += 2 if text[cursor] == "\\" else 1
    return min(cursor + 1, len(text))


def word_before(text: str, index: int) -> bool:
    """Whether the character before the index continues an identifier."""
    return index > 0 and (text[index - 1].isalnum() or text[index - 1] == "_")


@dataclass(frozen=True)
class Source:
    """Rust source with literals blanked out at their original positions."""

    clean: str

    @classmethod
    def from_rust(cls, text: str) -> "Source":
        """A source with every comment, string, and character literal blanked out."""
        characters = list(text)
        for span in literal_spans(text):
            for position in range(span.start, span.end):
                if characters[position] != "\n":
                    characters[position] = " "
        return cls(clean="".join(characters))

    def line_of(self, index: int) -> int:
        """The 1-based line number of a character index."""
        return self.clean.count("\n", 0, index) + 1


def doc_comments(text: str) -> Iterator[tuple[int, str]]:
    """Every doc comment, outer (`///`, `/** */`) or inner (`//!`, `/*! */`), with its start."""
    for span in literal_spans(text):
        comment = text[span.start:span.end]
        if span.kind != "comment":
            continue
        if comment.startswith("//!") or (comment.startswith("///") and not comment.startswith("////")):
            yield span.start, comment
            continue
        if comment.startswith("/*!") or (comment.startswith("/**") and comment[3:4] not in ("*", "/")):
            yield span.start, comment


def nearest_manifest(start: Path | None = None) -> Path | None:
    """The nearest Cargo.toml at or above a path, bounded by the project root, or None.

    Walking upward from the edited file (rather than testing the project root alone) is what
    lets the checks travel: a repository that keeps its manifest below the root still resolves
    its Rust context instead of silently disabling every Rust check.

    The walk stops at the project root and never substitutes the root manifest for a path
    outside it. A file the project does not contain is not governed by the project's rules, and
    binding this project's module tree to a foreign file would report violations against a
    crate that file never belonged to."""
    root = project_root().resolve()
    base = (start if start is not None else root).resolve()
    base = base if base.is_dir() else base.parent
    lineage = [base, *base.parents]

    if root not in lineage:
        return None

    for directory in lineage[:lineage.index(root) + 1]:
        manifest = directory / "Cargo.toml"

        if manifest.is_file():
            return manifest
    return None


def context_present(start: Path | None = None) -> bool:
    """Whether the Rust language context exists: a Cargo.toml at or above the path, and the
    path outside the directories Cargo writes. A crate the registry unpacks under .cargo-home/
    carries its own Cargo.toml and src/, and is not governed by this project's rules."""
    if start is not None and BUILD_DIRECTORIES.intersection(project_parts(start)):
        return False
    return nearest_manifest(start) is not None


def project_parts(path: Path) -> tuple[str, ...]:
    """The parts of a path below the project root, or all its parts when it lies outside."""
    resolved_path = path.resolve()
    try:
        return resolved_path.relative_to(project_root().resolve()).parts
    except ValueError:
        return resolved_path.parts


def module_path(path: Path) -> tuple[str, ...] | None:
    """The module path of a source file below the src/ of its nearest manifest, without the
    leading `crate`, or None when the file is not under that src/. `src/lib.rs` and `src/main.rs`
    are crate roots (an empty path), `x/mod.rs` and `x.rs` both name module `x`."""
    manifest = nearest_manifest(path)

    if manifest is None:
        return None
    try:
        relative = path.resolve().relative_to((manifest.parent / "src").resolve())
    except ValueError:
        return None
    parts = [*relative.parent.parts, relative.stem]

    if parts[-1] == "mod" or (len(parts) == 1 and parts[0] in ("lib", "main")):
        parts = parts[:-1]
    return tuple(parts)


def domain_crates(start: Path) -> frozenset[str] | None:
    """The crates the domain may depend on besides std and core, read from the DOMAIN_CRATES
    variable of the Makefile beside the nearest Cargo.toml (`jiff|thiserror`), or None when it
    cannot be read. Reading it from the Makefile keeps the hook and `make review` on one list."""
    manifest = nearest_manifest(start)

    if manifest is None:
        return None
    try:
        makefile = (manifest.parent / "Makefile").read_text(encoding="utf-8")
    except OSError:
        return None
    match = DOMAIN_CRATES_VARIABLE.search(makefile)

    if match is None:
        return None
    crates = frozenset(name.strip() for name in match.group(1).split("|") if name.strip())
    return crates or None


def dependency_crates(start: Path) -> frozenset[str]:
    """The crate names of the [dependencies] table of the nearest Cargo.toml, `-` read as `_`."""
    manifest = nearest_manifest(start)

    if manifest is None:
        return frozenset()
    try:
        table = DEPENDENCIES_TABLE.search(manifest.read_text(encoding="utf-8"))
    except OSError:
        return frozenset()
    if table is None:
        return frozenset()
    return frozenset(name.replace("-", "_") for name in DEPENDENCY_KEY.findall(table.group(1)))


def top_level_pieces(text: str) -> list[str]:
    """The comma-separated pieces of a list, ignoring separators nested in brackets."""
    pieces, depth, current = [], 0, []
    for character in text:
        if character in "([{":
            depth += 1

        if character in ")]}":
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


def expanded_use_tree(tree: str) -> list[str]:
    """Every leaf path of one use tree, aliases dropped: `a::{b, c::{self, d}}` gives
    `a::b`, `a::c`, and `a::c::d`."""
    tree = re.sub(r"\s+as\s+\w+", "", tree)
    tree = re.sub(r"\s+", "", tree).removeprefix("::")
    brace = tree.find("{")

    if brace == -1:
        return [tree] if tree else []
    prefix = tree[:brace]
    paths = []
    for piece in top_level_pieces(tree[brace + 1:tree.rfind("}")]):
        for leaf in expanded_use_tree(piece):
            paths.append(prefix.removesuffix("::") if leaf == "self" else prefix + leaf)
    return paths


@dataclass(frozen=True)
class Reference:
    """One path a file names: where it appears, how it was written, and what it resolves to.

    `own` is the absolute module path inside this crate (without `crate`) when the path is
    crate-relative, `self`- or `super`-relative, or through a child module. `external` is the
    first segment when the path names another crate. `escapes` marks a `super` chain that
    climbs above the crate root."""

    index: int
    written: str
    own: tuple[str, ...] | None
    external: str
    escapes: bool


def segments_after_super(segments: list[str]) -> tuple[str, ...]:
    """The segments of a path after its leading run of `super`."""
    leading = 0
    while leading < len(segments) and segments[leading] == "super":
        leading += 1
    return tuple(segments[leading:])


def resolved(written: str, module: tuple[str, ...], children: frozenset[str], index: int) -> Reference:
    """One written path resolved against the module it appears in."""
    segments = [segment for segment in written.split("::") if segment]
    head = segments[0] if segments else ""

    if head == "crate":
        return Reference(index=index, written=written, own=tuple(segments[1:]), external="", escapes=False)
    if head == "self":
        return Reference(index=index, written=written, own=module + tuple(segments[1:]), external="", escapes=False)
    if head == "super":
        leading = len(segments) - len(segments_after_super(segments))
        if leading > len(module):
            return Reference(index=index, written=written, own=(), external="", escapes=True)
        own = module[:len(module) - leading] + segments_after_super(segments)
        return Reference(index=index, written=written, own=own, external="", escapes=False)
    if head in children:
        return Reference(index=index, written=written, own=module + tuple(segments), external="", escapes=False)
    return Reference(index=index, written=written, own=None, external=head, escapes=False)


def bracket_spans(text: str) -> dict[int, int]:
    """Every opening bracket position mapped to its closing position."""
    spans: dict[int, int] = {}
    stack: list[int] = []

    for position, character in enumerate(text):
        if character in "([{":
            stack.append(position)

        if character in ")]}" and stack:
            spans[stack.pop()] = position

    return spans


def module_at(source: Source, module: tuple[str, ...], index: int) -> tuple[str, ...]:
    """The module a position belongs to, descending into every inline `mod name { ... }` around it."""
    spans = bracket_spans(source.clean)
    nested = list(module)
    for declaration in MODULE_DECLARATION.finditer(source.clean):
        if declaration.group(2) != "{":
            continue
        brace = declaration.end() - 1
        if brace < index <= spans.get(brace, brace):
            nested.append(declaration.group(1))
    return tuple(nested)


def references(source: Source, module: tuple[str, ...]) -> list[Reference]:
    """Every use-declaration leaf and every inline crate-, self-, or super-relative path of a
    source, resolved against the module each appears in."""
    children = frozenset(declaration.group(1) for declaration in MODULE_DECLARATION.finditer(source.clean))
    found = []
    covered = []
    for declaration in USE_DECLARATION.finditer(source.clean):
        covered.append((declaration.start(), declaration.end()))
        scope = module_at(source, module, declaration.start())
        for leaf in expanded_use_tree(declaration.group(1)):
            found.append(resolved(leaf, scope, children, declaration.start()))
    for inline in INLINE_OWN_PATH.finditer(source.clean):
        if any(start <= inline.start() < end for start, end in covered):
            continue
        written = re.sub(r"\s+", "", inline.group(0))
        found.append(resolved(written, module_at(source, module, inline.start()), children, inline.start()))
    return sorted(found, key=lambda reference: reference.index)
