use crate::domain::LinkService;

/// Represents common application state
pub struct AppState {
    /// `LinkService` implementation
    pub link_service: Box<dyn LinkService>,
}

impl AppState {
    /// Create a new `AppState` with `LinkService` and `server_url`
    pub fn new(link_service: Box<dyn LinkService>) -> Self {
        Self { link_service }
    }
}
