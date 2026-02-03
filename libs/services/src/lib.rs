pub mod auth;
pub mod mail;
pub mod templates;
pub mod cloudinary;

pub use auth::AuthServiceClient;
pub use mail::MailService;
pub use templates::*;
pub use cloudinary::CloudinaryService;

// Usage:
// let (html, text) = EmailTemplates::welcome_email(&user_name);
// mail_service.send_notification(&email, "Welcome", &html, Some(&text)).await?;
