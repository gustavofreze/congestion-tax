#!/usr/bin/env python3
# Vendored from payment (.claude/hooks/tests/test-php-context.py) on 2026-11-18. Adapted to this project: Rust context
# (Cargo.toml, module paths, the Makefile DOMAIN_CRATES allowlist), a section for the Cargo build directories and one
# for the lexer and use trees, and the real tree walked from src/ and tests/ instead of git ls-files.
"""Regression suite for Rust context resolution in the hooks.

Run it from anywhere: python3 .claude/hooks/tests/test-rust-context.py
Exit 0 means every check passed. Exit 1 lists the failures.

Every section carries a positive control, because a suite that only proves the hooks stay
quiet proves nothing: a hook that never fires passes every negative test.

What it pins down:
  A. Identity still resolves in this repository, from absolute AND relative paths, since a
     hook receives whichever the caller passed.
  B. A path outside the project root resolves to no manifest, so a foreign file never
     inherits this project's module tree and gets reported against a crate it never
     belonged to. A path under target/ or .cargo-home/ has no Rust context either.
  C. A manifest below the project root is found, with the Makefile beside it, which is the
     case the upward walk exists for.
  D. A repository with no manifest gates every Rust check off without output.
  E. A malformed or partial manifest or Makefile degrades to a skip, never a traceback.
  F. The lexer blanks comments, strings, and character literals but keeps lifetimes, and a
     use tree expands to every leaf it names.
  G. The real tree is still clean for the layer checks. The member ordering hook must run
     without a traceback, and the violations it reports on src/ are listed, not asserted.
"""
import subprocess
import sys
import tempfile
from pathlib import Path

HOOKS = Path(__file__).resolve().parent.parent
REPO = HOOKS.parent.parent
sys.path.insert(0, str(HOOKS))

from hooklib import rust  # noqa: E402

FOREIGN = '''use serde::Serialize;
use crate::driven::postgres::PostgresThings;

pub struct Thing;
'''

failures = []


def check(label, actual, expected):
    ok = actual == expected
    print(f"  {'PASS' if ok else 'FAIL'}  {label}")
    if not ok:
        print(f"        expected {expected!r}")
        print(f"        actual   {actual!r}")
        failures.append(label)


def run_hook(hook, path, project_dir, cwd):
    proc = subprocess.run(
        [sys.executable, '-B', str(HOOKS / f'{hook}.py'), str(path)],
        capture_output=True, text=True, cwd=str(cwd),
        env={'CLAUDE_PROJECT_DIR': str(project_dir), 'PATH': '/usr/bin:/bin', 'HOME': '/tmp'},
    )
    return proc.returncode, (proc.stdout + proc.stderr).strip()


def project(base, name, makefile='DOMAIN_CRATES := jiff\n', manifest='[package]\nname = "thing"\n'):
    root = Path(base) / name
    (root / 'src/application/domain/models').mkdir(parents=True)
    (root / 'src/application/handlers').mkdir(parents=True)
    if manifest is not None:
        (root / 'Cargo.toml').write_text(manifest)
    if makefile is not None:
        (root / 'Makefile').write_text(makefile)
    return root


print("=== A. in-tree resolution still works (positive control) ===")
import os  # noqa: E402

os.environ['CLAUDE_PROJECT_DIR'] = str(REPO)
os.chdir(REPO)

# Expected values are derived from this repository's own files, never written here, so the
# suite travels with the toolkit. Each one is read a DIFFERENT way than the implementation
# reads it, so the assertion cannot be a tautology.
makefile_line = next(line for line in (REPO / 'Makefile').read_text().splitlines()
                     if line.startswith('DOMAIN_CRATES'))
domain_crates = frozenset(makefile_line.split('=', 1)[1].strip().split('|'))
domain_file = next((REPO / 'src/application/domain').rglob('*.rs'))
domain_module = tuple(domain_file.relative_to(REPO / 'src').with_suffix('').parts)
print(f"  (derived from the Makefile: domain crates {sorted(domain_crates)!r})")

check("absolute src path finds the root Cargo.toml",
      rust.nearest_manifest(REPO / 'src'), REPO / 'Cargo.toml')
check("RELATIVE src path finds it too (paths arrive relative)",
      rust.nearest_manifest(Path('src')).resolve(), (REPO / 'Cargo.toml').resolve())
check("domain_crates matches the Makefile line",
      rust.domain_crates(Path('src')), domain_crates)
check("module_path of a domain file, absolute",
      rust.module_path(domain_file), domain_module)
check("module_path of the same file, relative",
      rust.module_path(domain_file.relative_to(REPO)), domain_module)
check("src/lib.rs is the crate root", rust.module_path(REPO / 'src/lib.rs'), ())
check("context_present true for an in-tree rust file",
      rust.context_present(Path('src')), True)

print()
print("=== B. out-of-tree and build paths no longer inherit this project's identity ===")
with tempfile.TemporaryDirectory() as tmp:
    out = Path(tmp) / 'outside/src/application/domain/models'
    out.mkdir(parents=True)
    (out / 'thing.rs').write_text(FOREIGN)

    check("nearest_manifest returns None for an out-of-tree file",
          rust.nearest_manifest(out / 'thing.rs'), None)
    check("context_present false for an out-of-tree file",
          rust.context_present(out / 'thing.rs'), False)
    check("module_path None for an out-of-tree file",
          rust.module_path(out / 'thing.rs'), None)

    rc, msg = run_hook('check-rust-domain-imports', out / 'thing.rs', REPO, REPO)
    check("domain-imports exits 0 on an out-of-tree file", rc, 0)
    check("domain-imports says nothing about it", msg, '')

    rc, msg = run_hook('check-rust-hexagonal-layers', out / 'thing.rs', REPO, REPO)
    check("hexagonal-layers exits 0 on an out-of-tree file", rc, 0)

for directory in sorted(rust.BUILD_DIRECTORIES):
    check(f"context_present false under {directory}/",
          rust.context_present(REPO / directory / 'registry/src/crate/src/lib.rs'), False)

print()
print("=== C. a manifest below the project root, with its own Makefile ===")
with tempfile.TemporaryDirectory() as tmp:
    mono = Path(tmp) / 'mono'
    service = project(mono / 'services', 'thing')
    target = service / 'src/application/domain/models/thing.rs'
    target.write_text(FOREIGN)

    rc, msg = run_hook('check-rust-domain-imports', target, mono, mono)
    check("domain-imports CATCHES the violation (exit 2)", rc, 2)
    check("and binds the sub-root's own allowlist, not this repository's",
          'the crates jiff, and' in msg, True)

    handler = service / 'src/application/handlers/thing_handler.rs'
    handler.write_text(FOREIGN)
    rc, msg = run_hook('check-rust-hexagonal-layers', handler, mono, mono)
    check("hexagonal-layers CATCHES application naming driven (exit 2)", rc, 2)
    check("and names the driven path", 'crate::driven::postgres::PostgresThings' in msg, True)

print()
print("=== D. non-Rust project gates off silently ===")
with tempfile.TemporaryDirectory() as tmp:
    go = Path(tmp) / 'goproj'
    (go / 'src/application/domain/models').mkdir(parents=True)
    (go / 'go.mod').write_text('module example.com/x\n')
    target = go / 'src/application/domain/models/thing.rs'
    target.write_text(FOREIGN)

    for hook in ('check-rust-domain-imports', 'check-rust-hexagonal-layers', 'check-rust-member-ordering'):
        rc, msg = run_hook(hook, target, go, go)
        check(f"{hook} exits 0 in a non-Rust repo and says nothing", (rc, msg), (0, ''))

print()
print("=== E. malformed and partial manifests degrade, never crash ===")
with tempfile.TemporaryDirectory() as tmp:
    for name, manifest, makefile in [
        ('broken', '[package\nname = ', 'DOMAIN_CRATES := jiff\n'),
        ('nomakefile', '[package]\nname = "x"\n', None),
        ('novariable', '[package]\nname = "x"\n', 'review:\n\ttrue\n'),
        ('emptyvariable', '[package]\nname = "x"\n', 'DOMAIN_CRATES :=\n'),
    ]:
        root = project(tmp, name, makefile=makefile, manifest=manifest)
        target = root / 'src/application/domain/models/thing.rs'
        target.write_text(FOREIGN)
        rc, msg = run_hook('check-rust-domain-imports', target, root, root)
        check(f"domain-imports survives a {name} project (exit 0 or 2, no traceback)",
              rc in (0, 2) and 'Traceback' not in msg, True)
        rc, msg = run_hook('check-rust-hexagonal-layers', target, root, root)
        check(f"hexagonal-layers survives a {name} project",
              rc in (0, 2) and 'Traceback' not in msg, True)

print()
print("=== F. the lexer and the use trees ===")
LEXED = '''// use crate::driven::a::A;
/* outer /* use crate::driven::b::B; */ still a comment */
const TEXT: &str = "use crate::driven::c::C;";
const RAW: &str = r#"use crate::driven::d::D; "quoted" "#;
const QUOTE: char = '"';
fn borrow<'a>(text: &'a str) -> &'a str { text }
use crate::driven::e::E;
'''
lexed = rust.Source.from_rust(LEXED)
written = [reference.written for reference in rust.references(lexed, ('application',))]
check("only the real use declaration is a reference (positive control)", written, ['crate::driven::e::E'])
check("the lifetime survives the blanking", "<'a>" in lexed.clean, True)
check("a use tree expands to every leaf, self and aliases included",
      rust.expanded_use_tree('crate::a::{b, c::{self, d as e}, f::*}'),
      ['crate::a::b', 'crate::a::c', 'crate::a::c::d', 'crate::a::f::*'])
check("super resolves against the module of the file",
      rust.resolved('super::Amount', ('application', 'domain', 'models', 'x'), frozenset(), 0).own,
      ('application', 'domain', 'models', 'Amount'))
check("a super chain above the crate root escapes",
      rust.resolved('super::super::X', ('driver',), frozenset(), 0).escapes, True)
check("a doc comment is found and a plain comment is not",
      [comment for _, comment in rust.doc_comments('/// doc\n// plain\n//// rule\n//! inner\n')],
      ['/// doc', '//! inner'])

print()
print("=== G. the real tree is still clean for the layer checks (positive control) ===")
sources = sorted(str(path.relative_to(REPO)) for root in ('src', 'tests') for path in (REPO / root).rglob('*.rs'))
print(f"  ({len(sources)} rust files under src/ and tests/)")
check("at least one rust file exists", bool(sources), True)
for hook in ('check-rust-domain-imports', 'check-rust-hexagonal-layers'):
    proc = subprocess.run([sys.executable, '-B', str(HOOKS / f'{hook}.py'), *sources],
                          capture_output=True, text=True, cwd=str(REPO),
                          env={'CLAUDE_PROJECT_DIR': str(REPO), 'PATH': '/usr/bin:/bin', 'HOME': '/tmp'})
    check(f"{hook} clean on the real tree", (proc.returncode, proc.stderr.strip()), (0, ''))
proc = subprocess.run([sys.executable, '-B', str(HOOKS / 'check-rust-member-ordering.py'), *sources],
                      capture_output=True, text=True, cwd=str(REPO),
                      env={'CLAUDE_PROJECT_DIR': str(REPO), 'PATH': '/usr/bin:/bin', 'HOME': '/tmp'})
check("check-rust-member-ordering runs on the real tree without a traceback",
      proc.returncode in (0, 2) and 'Traceback' not in proc.stderr, True)
reported = [line for line in proc.stderr.splitlines() if line]
print(f"  INFO  check-rust-member-ordering reports {len(reported)} parameter-order violations on the real tree")

print()
print("=" * 62)
print("FAILURES:", failures or "none")
sys.exit(1 if failures else 0)
