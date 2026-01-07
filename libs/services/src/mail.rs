use anyhow::Result;
use resend_rs::{types::CreateEmailBaseOptions, Resend};
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
        reset_token: &str,
        frontend_url: &str,
    ) -> Result<()> {
        let reset_link = format!("{}/reset-password?token={}", frontend_url, reset_token);
        
        let subject = "Reset Your Password - HealthBridge";
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
                    .button {{ 
                        display: inline-block; 
                        padding: 12px 24px; 
                        background-color: #4F46E5; 
                        color: white; 
                        text-decoration: none; 
                        border-radius: 6px; 
                        margin: 20px 0;
                    }}
                    .footer {{ padding: 20px; text-align: center; color: #6b7280; font-size: 12px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>Password Reset Request</h1>
                    </div>
                    <div class="content">
                        <p>Hi {},</p>
                        <p>We received a request to reset your password for your HealthBridge account.</p>
                        <p>Click the button below to reset your password:</p>
                        <p style="text-align: center;">
                            <a href="{}" class="button">Reset Password</a>
                        </p>
                        <p>Or copy and paste this link into your browser:</p>
                        <p style="word-break: break-all; color: #4F46E5;">{}</p>
                        <p><strong>This link will expire in 1 hour.</strong></p>
                        <p>If you didn't request a password reset, you can safely ignore this email.</p>
                    </div>
                    <div class="footer">
                        <p>© 2024 HealthBridge. All rights reserved.</p>
                        <p>This is an automated message, please do not reply.</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            user_name, reset_link, reset_link
        );

        let text_body = format!(
            "Hi {},\n\n\
            We received a request to reset your password for your HealthBridge account.\n\n\
            Click the link below to reset your password:\n\
            {}\n\n\
            This link will expire in 1 hour.\n\n\
            If you didn't request a password reset, you can safely ignore this email.\n\n\
            Best regards,\n\
            The HealthBridge Team",
            user_name, reset_link
        );

        let email = CreateEmailBaseOptions::new(
            &self.from_email,
            vec![to_email],
            subject,
        )
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

    pub async fn send_notification(
        &self,
        to_email: &str,
        subject: &str,
        html_body: &str,
        text_body: Option<&str>,
    ) -> Result<()> {
        let mut email = CreateEmailBaseOptions::new(
            &self.from_email,
            vec![to_email],
            subject,
        )
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