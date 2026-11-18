<!-- Vendored from payment (.claude/skills/php-creating-application/references/typed-collection.md) on 2026-11-18. Adapted to this project: the subclass of the domain Collection wrapper became a newtype over Vec with FromIterator, worked from src/application/domain/models/tax_rules/passages.rs. -->

# Typed collection

Use this when adding a typed collection of a domain element, a newtype over a `Vec` that names the element type and owns
the operations the domain needs over the whole set.

```rust
// src/application/domain/models/tax_rules/passages.rs
//
// Typed collection worked example. A tuple struct over Vec<Element> with a private field, built
// through new() over any IntoIterator and through FromIterator, so collect() produces it. The
// constructor puts the elements in the order the domain needs (chronological here) once, so every
// operation can rely on it. It exposes only the operations the domain asks for (by_day), never the
// Vec itself. The name is the plural of the concept (Passages), never a technical suffix like Items,
// List, Set, or Collection.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Passages(Vec<Passage>);

impl Passages {
    #[must_use]
    pub fn new(passages: impl IntoIterator<Item = Passage>) -> Self {
        let mut chronological: Vec<Passage> = passages.into_iter().collect();
        chronological.sort_unstable();
        Self(chronological)
    }

    pub(super) fn by_day(&self) -> impl Iterator<Item = (Date, &[Passage])> {
        self.0
            .chunk_by(|current, next| current.date() == next.date())
            .filter_map(|day_passages| day_passages.first().map(|first| (first.date(), day_passages)))
    }
}

impl FromIterator<Passage> for Passages {
    fn from_iter<I: IntoIterator<Item = Passage>>(passages: I) -> Self {
        Self::new(passages)
    }
}
```
