use crate::domain::LinkService;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;

/// Represents common application state
pub struct AppState {
    /// `LinkService` implementation
    pub link_service: Box<dyn LinkService>,

    /// Prometheus handle
    pub prometheus_handle: PrometheusHandle,
}

impl AppState {
    /// Create a new `AppState` with `LinkService` and `prometheus_handle`
    pub fn new(link_service: Box<dyn LinkService>, prometheus_handle: PrometheusHandle) -> Self {
        Self {
            link_service,
            prometheus_handle,
        }
    }
}
