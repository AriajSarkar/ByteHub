use std::sync::Arc;
use twilight_http::Client;
use twilight_model::id::{Id, marker::ApplicationMarker};

#[derive(Clone)]
pub struct DiscordClient {
    pub http: Arc<Client>,
    pub application_id: Id<ApplicationMarker>,
}

impl DiscordClient {
    pub fn new(token: &str, application_id: u64) -> Self {
        let http = Arc::new(Client::new(token.to_string()));
        Self {
            http,
            application_id: Id::new(application_id),
        }
    }
}
