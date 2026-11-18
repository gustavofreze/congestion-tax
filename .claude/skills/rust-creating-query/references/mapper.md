<!-- Vendored from payment (.claude/skills/php-creating-query/references/mapper.md) on 2026-11-18. Adapted to this project: the PHP Data Mapper class over a DBAL row became the mapping inside the sqlx database adapter, worked from src/query/health/database/postgres_database_probe.rs, with the typed sqlx decode named for a row of columns. -->

# Read-model mapper

Use when turning what the database answered into the typed read model, the only place on the read side that reads a raw
row or a raw outcome.

The raw answer of sqlx (a `PgRow`, a scalar, a `Result` of the statement) is the adapter's private input and never
travels further. The adapter decodes it into the read model and answers that. A row decodes through a typed call
(`sqlx::query_scalar` for one column, `sqlx::query_as` into a tuple for several), never by reading columns by name in
more than one place. The `sqlx` features in `Cargo.toml` carry no derive, and this project reads its one multi-table
shape as a single JSON document column decoded with serde (`PostgresTaxRulesCatalog` reads `Json<DecodedTaxRules>`).
A decoded struct is a type, so it gets its own file under the slice's `database/` module, named after what it holds,
never `Record` (the row is the input, not the role).

```rust
// ---------------------------------------------------------------------------------------------
// src/query/health/database/postgres_database_probe.rs
//
// The adapter runs the statement its query constant holds, then builds the read model from the
// outcome in one place: the failure, when there is one, becomes the detail, and the elapsed time
// and the component name fill the rest. Nothing past this function sees the sqlx outcome.

const PROBE_DATABASE: &str = r"SELECT 1";

pub struct PostgresDatabaseProbe {
    pool: PgPool,
}

impl PostgresDatabaseProbe {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl DependencyProbe for PostgresDatabaseProbe {
    async fn probe(&self) -> DependencyStatus {
        let started_at = Instant::now();
        let outcome = sqlx::query(PROBE_DATABASE).execute(&self.pool).await;

        DependencyStatus {
            detail: outcome.err().map(|failure| failure.to_string()),
            elapsed: started_at.elapsed(),
            component: "database".to_owned(),
        }
    }
}
```
