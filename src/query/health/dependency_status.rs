use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyStatus {
    pub detail: Option<String>,
    pub elapsed: Duration,
    pub component: String,
}
