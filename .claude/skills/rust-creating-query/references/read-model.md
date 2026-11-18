<!-- Vendored from payment (.claude/skills/php-creating-query/references/read-model.md) on 2026-11-18. Adapted to this project: the PHP read model with toArray() became a read model of primitives plus a Serialize response, worked from DependencyStatus and DependencyStatusResponse of src/query/health/, and the generic Money and Reference sub-DTOs were dropped because no read here carries them. -->

# Read model

Use when shaping the read model of a slice and the response that owns its JSON wire envelope.

## Contents

- `DependencyStatus`: the read model the query trait answers.
- `DependencyStatusResponse`: the response that owns the wire envelope.

```rust
// The read model and its response. The read model is strongly typed: primitives plus typed nested
// read models for fixed-shape nested data, never a loose serde_json::Value for a known shape. A
// Value is allowed ONLY for genuinely dynamic or polymorphic output. The response is the single
// owner of the JSON wire envelope: a #[derive(Serialize)] struct with private fields, built from
// the read model through From. It whitelists the wire shape: it names every field the response
// emits, so a column added to the query or a field added to the read model stays invisible to the
// client until the response lists it. This is a whitelist by construction, never a blacklist that
// leaks a newly added sensitive field the moment someone forgets to exclude it.
// The read model lives at the slice root, src/query/<context>/<read_model>.rs.
// The response lives in the slice's http/ module, src/query/<context>/http/<name>_response.rs.

// ---------------------------------------------------------------------------------------------
// src/query/health/dependency_status.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyStatus {
    pub detail: Option<String>,
    pub elapsed: Duration,
    pub component: String,
}

// ---------------------------------------------------------------------------------------------
// src/query/health/http/dependency_status_response.rs

#[derive(Serialize)]
pub(super) struct DependencyStatusResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    status: &'static str,
    critical: bool,
    component: String,
    duration_in_milliseconds: f64,
}

impl From<DependencyStatus> for DependencyStatusResponse {
    fn from(dependency: DependencyStatus) -> Self {
        let status = match dependency.detail {
            Some(_) => "DOWN",
            None => "UP",
        };

        Self {
            detail: dependency.detail,
            status,
            critical: true,
            component: dependency.component,
            duration_in_milliseconds: (dependency.elapsed.as_secs_f64() * 100_000.0).round() / 100.0,
        }
    }
}
```
