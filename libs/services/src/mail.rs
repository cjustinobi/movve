use crate::templates::EmailTemplates;
use anyhow::Result;
use resend_rs::{Resend, types::CreateEmailBaseOptions};
use tracing::{error, info};

#[derive(Clone)]
pub struct MailService {
    client: Resend,
    from_email: String,
}

impl MailService {
    pub fn new(api_key: String, from_email: String) -> Self {
        let client = Resend::new(&api_key);
        Self { client, from_email }
    }

    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        user_name: &str,
        reset_code: &str,
    ) -> Result<()> {
        let subject = "Reset Your Password - Movve";
        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
                    .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
                    .header {{ background-color: #4F46E5; color: white; padding: 20px; text-align: center; }}
                    .content {{ padding: 30px; background-color: #f9fafb; }}
                    .code {{ 
                        display: block; 
                        padding: 20px; 
                        background-color: #EEF2FF; 
                        color: #4F46E5; 
                        font-size: 32px; 
                        font-weight: bold; 
                        text-align: center; 
                        letter-spacing: 5px;
                        border-radius: 8px;
                        margin: 20px 0;
                    }}
                    .footer {{ padding: 20px; text-align: center; color: #6b7280; font-size: 12px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>Password Reset Code</h1>
                    </div>
                    <div class="content">
                        <p>Hi {},</p>
                        <p>We received a request to reset your password for your Movve account.</p>
                        <p>Use the following code to reset your password:</p>
                        <div class="code">{}</div>
                        <p><strong>This code will expire in 30 minutes.</strong></p>
                        <p>If you didn't request a password reset, you can safely ignore this email.</p>
                    </div>
                    <div class="footer">
                        <p>© Movve. All rights reserved.</p>
                        <p>This is an automated message, please do not reply.</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            user_name, reset_code
        );

        let text_body = format!(
            "Hi {},\n\n\
            We received a request to reset your password for your Movve account.\n\n\
            Use the following code to reset your password:\n\
            {}\n\n\
            This code will expire in 30 minutes.\n\n\
            If you didn't request a password reset, you can safely ignore this email.\n\n\
            Best regards,\n\
            The Movve Team",
            user_name, reset_code
        );

        let email = CreateEmailBaseOptions::new(&self.from_email, vec![to_email], subject)
            .with_html(&html_body)
            .with_text(&text_body);

        match self.client.emails.send(email).await {
            Ok(_) => {
                info!("Password reset email sent to: {}", to_email);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send email to {}: {:?}", to_email, e);
                Err(anyhow::anyhow!("Failed to send email: {}", e))
            }
        }
    }

    pub async fn send_welcome_email(
        &self,
        to_email: &str,
        user_name: &str,
        verification_code: &str,
    ) -> Result<()> {
        let (subject, html, text) = EmailTemplates::welcome_email(user_name, verification_code);
        self.send_notification(to_email, &subject, &html, Some(&text))
            .await
    }

    pub async fn send_verification_code_email(
        &self,
        to_email: &str,
        user_name: &str,
        code: &str,
    ) -> Result<()> {
        let (subject, html, text) = EmailTemplates::verification_code_email(user_name, code);
        self.send_notification(to_email, &subject, &html, Some(&text))
            .await
    }

    pub async fn send_notification(
        &self,
        to_email: &str,
        subject: &str,
        html_body: &str,
        text_body: Option<&str>,
    ) -> Result<()> {
        let mut email = CreateEmailBaseOptions::new(&self.from_email, vec![to_email], subject)
            .with_html(html_body);

        if let Some(text) = text_body {
            email = email.with_text(text);
        }

        match self.client.emails.send(email).await {
            Ok(_) => {
                info!("Notification email sent to: {}", to_email);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send notification to {}: {:?}", to_email, e);
                Err(anyhow::anyhow!("Failed to send notification: {}", e))
            }
        }
    }
}
