pub struct EmailTemplates;

impl EmailTemplates {
    pub fn welcome_email(user_name: &str) -> (String, String, String) {
        let subject = "Welcome to Movve!".to_string();
        let html = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <body style="font-family: Arial, sans-serif;">
                <h1>Welcome to Movve, {}!</h1>
                <p>Thank you for joining us. Get started by exploring our services.</p>
            </body>
            </html>
            "#,
            user_name
        );
        let text = format!("Welcome to Movve, {}!\n\nThank you for joining us.", user_name);
        (subject, html, text)
    }

    pub fn ride_confirmation(rider_name: &str, driver_name: &str, pickup_location: &str) -> (String, String) {
        let _subject = "Ride Confirmed";
        let html = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <body style="font-family: Arial, sans-serif;">
                <h1>Your ride is confirmed!</h1>
                <p>Hi {},</p>
                <p>Your driver {} will pick you up at {}</p>
            </body>
            </html>
            "#,
            rider_name, driver_name, pickup_location
        );
        let text = format!(
            "Your ride is confirmed!\n\nHi {},\n\nYour driver {} will pick you up at {}",
            rider_name, driver_name, pickup_location
        );
        (html, text)
    }

    pub fn verification_email(
        user_name: &str,
        verification_link: &str,
    ) -> (String, String, String) {
        let subject = "Verify your email address".to_string();
        let html = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <body style="font-family: Arial, sans-serif;">
                <h1>Verify your email address</h1>
                <p>Hi {},</p>
                <p>Thank you for signing up with Movve. Please verify your email address by clicking the link below:</p>
                <p><a href="{}">Verify Email</a></p>
            </body>
            </html>
            "#,
            user_name, verification_link
        );
        let text = format!(
            "Verify your email address\n\nHi {},\n\nThank you for signing up with Movve. Please verify your email address by clicking the link below:\n\n{}",
            user_name, verification_link
        );
        (subject, html, text)
    }
}