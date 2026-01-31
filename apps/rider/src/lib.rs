pub mod docs;
pub mod handlers;
pub mod model;
pub mod repository;
pub mod routes;
pub mod schema; 
pub mod service;

use service::RiderService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub rider_service: Arc<RiderService>,
    pub jwt_secret: String,
}
