pub mod auth;
pub mod mail;
pub mod templates;

pub use auth::AuthServiceClient;
pub use mail::MailService;
pub use templates::*;

// Usage:
// let (html, text) = EmailTemplates::welcome_email(&user_name);
// mail_service.send_notification(&email, "Welcome", &html, Some(&text)).await?;
