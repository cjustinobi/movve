use common::AppError;
use regex::Regex;

/// Validate email format
pub fn validate_email(email: &str) -> Result<(), AppError> {

    let email_regex = Regex::new(
        r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$"
    ).unwrap();

    if email_regex.is_match(email) {
        Ok(())
    } else {
        Err(AppError::BadRequest("Invalid email format".to_string()))
    }
}

/// Validate phone number length (digits only)
pub fn validate_phone_length(
    phone: &str,
    min_len: usize,
    max_len: usize,
) -> Result<(), AppError> {
    let digits_only = phone.chars().all(|c| c.is_ascii_digit());

    if !digits_only {
        return Err(AppError::BadRequest("Phone number must contain digits only".to_string()));
    }

    let len = phone.len();

    if len < min_len || len > max_len {
        return Err(AppError::BadRequest(format!("Phone number must be between {} and {} digits long", min_len, max_len)));
    }

    Ok(())
}
