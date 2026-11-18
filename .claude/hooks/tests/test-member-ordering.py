#!/usr/bin/env python3
# Vendored from payment (.claude/hooks/tests/test-member-ordering.py) on 2026-11-18. Adapted to this project: the
# Rust hook, snake_case start and end pairs, Rust fn signatures for PHP ones, no call-site section because Rust has no
# named argument, and a section for the contract-fixed signatures the Rust hook exempts.
"""Regression suite for the semantic pairs and exemptions of the member ordering hook.

Run it from anywhere: python3 .claude/hooks/tests/test-member-ordering.py
Exit 0 means every check passed. Exit 1 lists the failures.

Every section carries a positive control, because a suite that only proves the hook stays
quiet proves nothing: a hook that never fires passes every negative test.

What it pins down:
  A. A start and its end order start first, whatever their prefix or suffix, even where
     length or alphabet would put the end first.
  B. The same pair written end first is flagged at a declaration.
  C. A pair member without its partner orders by length and alphabet like any name.
  D. A genuine length misordering with no pair involved is still flagged, and the receiver
     is never counted as a parameter.
  E. The signatures a contract fixes are exempt: a trait impl method, an axum handler, a
     destructuring pattern, and an rstest test. The trait declaration itself is checked.
"""
import importlib.util
import subprocess
import sys
import tempfile
from pathlib import Path

HOOKS = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HOOKS))

specification = importlib.util.spec_from_file_location(
    'member_ordering', HOOKS / 'check-rust-member-ordering.py')
member_ordering = importlib.util.module_from_spec(specification)
specification.loader.exec_module(member_ordering)

ordering = member_ordering.PARAMETER_ORDERING

failures = []


def check(label, actual, expected):
    ok = actual == expected
    print(f"  {'PASS' if ok else 'FAIL'}  {label}")
    if not ok:
        print(f"        expected {expected!r}")
        print(f"        actual   {actual!r}")
        failures.append(label)


def run_hook(project, body):
    target = project / 'src/application/domain/models/slot.rs'
    target.write_text(body)
    proc = subprocess.run(
        [sys.executable, '-B', str(HOOKS / 'check-rust-member-ordering.py'), str(target)],
        capture_output=True, text=True, cwd=str(project),
        env={'CLAUDE_PROJECT_DIR': str(project), 'PATH': '/usr/bin:/bin', 'HOME': '/tmp'},
    )
    return proc.returncode, (proc.stdout + proc.stderr).strip()


def rust_struct(signature):
    return f'''pub struct Slot;

impl Slot {{
    pub fn new({signature}) -> Self {{
        Self
    }}
}}
'''


START_END_PAIRS = [
    ('start', 'end'),
    ('starts_at', 'ends_at'),
    ('starts_on', 'ends_on'),
    ('start_time', 'end_time'),
    ('window_start', 'window_end'),
    ('desired_start_time', 'desired_end_time'),
]

print("=== A. a start and its end order start first (positive control) ===")
for start, end in START_END_PAIRS:
    check(f"({start}, {end}) stays start first",
          ordering.sorted([end, start]), [start, end])
check("the pair sorts as a unit at the start key, among other names",
      ordering.sorted(['end_time', 'organization_id', 'id', 'start_time']),
      ['id', 'start_time', 'end_time', 'organization_id'])

print()
print("=== B. the hook flags an end-first pair and passes a start-first one ===")
with tempfile.TemporaryDirectory() as tmp:
    project = Path(tmp) / 'thing'
    (project / 'src/application/domain/models').mkdir(parents=True)
    (project / 'Cargo.toml').write_text('[package]\nname = "thing"\n')

    rc, msg = run_hook(project, rust_struct('starts_at: Time, ends_at: Time'))
    check("start-first declaration exits 0", (rc, msg), (0, ''))

    rc, msg = run_hook(project, rust_struct('desired_start_time: Time, desired_end_time: Time'))
    check("start-first prefixed pair exits 0", (rc, msg), (0, ''))

    rc, msg = run_hook(project, rust_struct('ends_at: Time, starts_at: Time'))
    check("end-first declaration is flagged (exit 2)", rc, 2)
    check("and the message requires start first",
          'required (starts_at, ends_at)' in msg, True)

    print()
    print("=== C. a lone pair member orders by length and alphabet ===")
    check("starts_at without ends_at falls back to length",
          ordering.sorted(['starts_at', 'id']), ['id', 'starts_at'])

    print()
    print("=== D. a genuine misordering without a pair is still flagged ===")
    rc, msg = run_hook(project, rust_struct('organization_id: u32, id: u32'))
    check("length misordering is flagged (exit 2)", rc, 2)
    check("and the message requires the shorter name first",
          'required (id, organization_id)' in msg, True)

    rc, msg = run_hook(project, rust_struct('&self, city: &City'))
    check("the receiver is not a parameter", (rc, msg), (0, ''))

    rc, msg = run_hook(project, rust_struct('mut organization_id: u32, id: impl Into<HashMap<u32, u32>>'))
    check("a mut binding and a generic type with a comma are still read (exit 2)", rc, 2)

    rc, msg = run_hook(project, rust_struct('currency: Currency, periods: impl IntoIterator<Item = TaxPeriod>'))
    check("a trailing impl IntoIterator parameter stays last (exit 0)", (rc, msg), (0, ''))

    rc, msg = run_hook(project, rust_struct(
        'flag: bool, months: impl IntoIterator<Item = Month>, holidays: impl IntoIterator<Item = Date>'))
    check("the impl IntoIterator parameters order among themselves (exit 0)", (rc, msg), (0, ''))

    rc, msg = run_hook(project, rust_struct('periods: impl IntoIterator<Item = TaxPeriod>, currency: Currency'))
    check("an impl IntoIterator parameter before a fixed one is flagged (exit 2)", rc, 2)
    check("and the message requires it last", 'required (currency, periods)' in msg, True)

    print()
    print("=== E. contract-fixed signatures are exempt, owned contracts are not ===")
    rc, msg = run_hook(project, '''pub struct Slot;

impl Display for Slot {
    fn fmt(&self, formatter: &mut Formatter<'_>, organization_id: u32) -> Result {
        Ok(())
    }
}
''')
    check("a trait impl method is exempt", (rc, msg), (0, ''))

    rc, msg = run_hook(project, '''pub trait Slots {
    fn reserve(&self, organization_id: u32, id: u32);
}
''')
    check("a trait declaration of ours is checked (exit 2)", rc, 2)

    rc, msg = run_hook(project, '''pub async fn reserve(service: State<Arc<Slots>>, body: Json<Request>) -> Response {
    todo()
}
''')
    check("an axum handler is exempt", (rc, msg), (0, ''))

    rc, msg = run_hook(project, '''fn reserve((organization_id, id): (u32, u32), slot: u32) {}
''')
    check("a destructuring pattern is exempt", (rc, msg), (0, ''))

    rc, msg = run_hook(project, '''#[rstest]
#[case(1, 2)]
fn reserve_when_free_then_reserved(#[case] organization_id: u32, #[case] id: u32) {}
''')
    check("an rstest test is exempt", (rc, msg), (0, ''))

print()
print("=" * 62)
print("FAILURES:", failures or "none")
sys.exit(1 if failures else 0)
