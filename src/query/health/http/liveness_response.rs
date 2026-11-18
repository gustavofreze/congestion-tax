use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct LivenessResponse {
    status: &'static str,
}

impl LivenessResponse {
    pub(super) const fn up() -> Self {
        Self { status: "UP" }
    }
}
