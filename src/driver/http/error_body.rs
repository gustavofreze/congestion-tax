use serde::Serialize;

#[derive(Serialize)]
pub(super) struct ErrorBody {
    code: &'static str,
    message: String,
}

impl ErrorBody {
    pub(super) const fn new(code: &'static str, message: String) -> Self {
        Self { code, message }
    }
}
