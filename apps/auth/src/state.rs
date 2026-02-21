use crate::service::{AuthService, RatingService};
use services::{CloudinaryService, DriverServiceClient, MailService};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub mail_service: Arc<MailService>,
    pub cloudinary_service: Arc<CloudinaryService>,
    pub driver_service: Arc<DriverServiceClient>,
    pub rating_service: Arc<RatingService>,
}
