pub mod auth;
pub mod cloudinary;
pub mod driver_client;
pub mod mail;
pub mod rider;
pub mod templates;

pub use auth::AuthServiceClient;
pub use cloudinary::CloudinaryService;
pub use driver_client::DriverServiceClient;
pub use mail::MailService;
pub use rider::RiderServiceClient;
pub use templates::*;

// Usage:
// let (html, text) = EmailTemplates::welcome_email(&user_name);
// mail_service.send_notification(&email, "Welcome", &html, Some(&text)).await?;
