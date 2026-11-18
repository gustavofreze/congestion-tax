use super::DependencyStatus;

/// Dependency the service cannot handle requests without.
pub trait DependencyProbe: Send + Sync {
    /// Probes the dependency and reports whether it answers, how long it took to answer, and why it failed when it did
    /// not.
    fn probe(&self) -> impl Future<Output = DependencyStatus> + Send;
}
