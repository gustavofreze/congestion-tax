use serde::Serialize;

#[derive(Serialize)]
pub(super) struct ErrorBody {
    code: &'static str,
    message: &'static str,
}

impl ErrorBody {
    pub(super) const fn new(code: &'static str, message: &'static str) -> Self {
        Self { code, message }
    }
}
