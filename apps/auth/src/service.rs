use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::Utc;
use common::{AppError, JwtConfig};
use jsonwebtoken::{EncodingKey, Header, encode};
use tracing::{error, info, instrument};
use utils::validate_phone_length;
use uuid::Uuid;

use crate::{
    model::{
        AuthResponse, Claims, CreateRatingRequest, LoginRequest, NewRating, Rating,
        RegisterRequest, UpdateProfileRequest, User, UserInfo,
    },
    repository::{RatingRepository, UserRepository, UserUpdate},
};

pub struct AuthService {
    repo: UserRepository,
    pub jwt_config: JwtConfig,
}

impl AuthService {
    pub fn new(repo: UserRepository, jwt_config: JwtConfig) -> Self {
        Self { repo, jwt_config }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<(AuthResponse, String), AppError> {
        utils::validate_email(&req.email)?;
        if self
            .repo
            .find_by_email(&req.email)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .is_some()
        {
            return Err(AppError::Conflict("Email already exists".to_string()));
        }

        let password_hash = self.hash_password(&req.password)?;

        let user = self
            .repo
            .create_user(&req.email, &password_hash, req.role)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // Create verification code
        let code = utils::generate_numeric_code(4);
        let expires_at = Utc::now() + chrono::Duration::minutes(15);
        self.repo
            .create_verification_code(user.id, &code, expires_at.naive_utc())
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let token = self.generate_token(&user)?;
        let refresh_token = self.generate_refresh_token(&user).await?;

        Ok((
            AuthResponse {
                token,
                refresh_token,
                user: UserInfo {
                    id: user.id,
                    email: user.email,
                    phone: user.phone,
                    first_name: user.first_name,
                    last_name: user.last_name,
                    role: user.role,
                    avatar: user.avatar,
                    gender: user.gender,
                    dob: user.dob,
                    nok_name: user.nok_name,
                    nok_phone: user.nok_phone,
                    email_verified: user.email_verified,
                    profile_completed: user.profile_completed,
                    profile: user.profile.clone(),
                },
            },
            code,
        ))
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, AppError> {
        utils::validate_email(&req.email)?;
        let user = self
            .repo
            .find_by_email(&req.email)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

        self.verify_password(&req.password, &user.password_hash)?;

        let token = self.generate_token(&user)?;
        let refresh_token = self.generate_refresh_token(&user).await?;

        Ok(AuthResponse {
            token,
            refresh_token,
            user: UserInfo {
                id: user.id,
                email: user.email,
                phone: user.phone,
                first_name: user.first_name,
                last_name: user.last_name,
                role: user.role,
                avatar: user.avatar,
                gender: user.gender,
                dob: user.dob,
                nok_name: user.nok_name,
                nok_phone: user.nok_phone,
                email_verified: user.email_verified,
                profile_completed: user.profile_completed,
                profile: user.profile.clone(),
            },
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        let claims = jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(self.jwt_config.secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?
        .claims;

        Ok(claims)
    }

    // ---------- Forgot Password ----------

    #[instrument(skip(self), fields(email = %email))]
    pub async fn forgot_password(&self, email: &str) -> Result<String, AppError> {
        info!("Starting forgot_password process");

        // 1️⃣ Find user by email
        let user = match self.repo.find_by_email(email).await {
            Ok(Some(user)) => {
                info!(user_id = %user.id, "User found");
                user
            }
            Ok(None) => {
                error!("User not found for email: {}", email);
                return Err(AppError::NotFound("User not found".to_string()));
            }
            Err(e) => {
                error!(error = ?e, "Database error while finding user");
                return Err(AppError::InternalError(e.to_string()));
            }
        };

        // 2️⃣ Create 4-digit password reset code
        let code = utils::generate_numeric_code(4);
        match self.repo.create_password_reset(user.id, &code).await {
            Ok(token) => {
                info!(user_id = %user.id, token = %token, "Password reset code created successfully");
                Ok(token)
            }
            Err(e) => {
                error!(user_id = %user.id, error = ?e, "Failed to create password reset code");
                Err(AppError::InternalError(e.to_string()))
            }
        }
    }

    // ---------- Refresh Tokens ----------
    pub async fn generate_refresh_token(&self, user: &User) -> Result<String, AppError> {
        let token = Uuid::new_v4().to_string();
        let expires_at =
            Utc::now() + chrono::Duration::hours(self.jwt_config.refresh_expiration_hours);

        self.repo
            .create_refresh_token(user.id, &token, expires_at.naive_utc())
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(token)
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<(), AppError> {
        self.repo
            .revoke_refresh_token(refresh_token)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))
    }

    pub async fn refresh_tokens(&self, refresh_token: &str) -> Result<AuthResponse, AppError> {
        // 1. Find and validate refresh token
        let rt = self
            .repo
            .find_refresh_token(refresh_token)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| {
                AppError::Unauthorized("Invalid or expired refresh token".to_string())
            })?;

        // 2. Get user
        let user = self
            .repo
            .find_by_id(rt.user_id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 3. Revoke old token
        self.repo
            .revoke_refresh_token(refresh_token)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 4. Generate new tokens
        let token = self.generate_token(&user)?;
        let new_refresh_token = self.generate_refresh_token(&user).await?;

        Ok(AuthResponse {
            token,
            refresh_token: new_refresh_token,
            user: UserInfo {
                id: user.id,
                email: user.email,
                phone: user.phone,
                first_name: user.first_name,
                last_name: user.last_name,
                role: user.role,
                avatar: user.avatar,
                gender: user.gender,
                dob: user.dob,
                nok_name: user.nok_name,
                nok_phone: user.nok_phone,
                email_verified: user.email_verified,
                profile_completed: user.profile_completed,
                profile: user.profile.clone(),
            },
        })
    }

    // ---------- Update Password ----------
    pub async fn update_password(
        &self,
        user_id: Uuid,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        let user = self
            .repo
            .find_by_id(user_id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        self.verify_password(old_password, &user.password_hash)?;

        let new_hash = self.hash_password(new_password)?;
        self.repo
            .update_password(user_id, &new_hash)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(())
    }

    // ---------- Email Verification ----------
    pub async fn resend_verification_code(&self, email: &str) -> Result<String, AppError> {
        let user = self
            .repo
            .find_by_email(email)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if user.email_verified {
            return Err(AppError::BadRequest("Email already verified".to_string()));
        }

        let code = utils::generate_numeric_code(4);
        let expires_at = Utc::now() + chrono::Duration::minutes(15);

        self.repo
            .create_verification_code(user.id, &code, expires_at.naive_utc())
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(code)
    }

    pub async fn verify_email(&self, email: &str, code: &str) -> Result<(), AppError> {
        // 1. Check if user is already verified
        let user = self
            .repo
            .find_by_email(email)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if user.email_verified {
            return Err(AppError::BadRequest("Email already verified".to_string()));
        }

        // 2. Find valid verification code
        let token = self
            .repo
            .find_verification_code(user.id, code)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| {
                AppError::BadRequest("Invalid or expired verification code".to_string())
            })?;

        // 3. Mark code as used
        self.repo
            .mark_verification_code_as_used(token.id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // 4. Mark user as verified
        self.repo
            .mark_user_as_verified(user.id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(())
    }

    pub async fn update_profile(
        &self,
        user_id: Uuid,
        req: UpdateProfileRequest,
    ) -> Result<User, AppError> {
        if let Some(ref phone) = req.phone {
            validate_phone_length(phone, 10, 15)?;
        }
        if let Some(ref nok_phone) = req.nok_phone {
            validate_phone_length(nok_phone, 10, 15)?;
        }
        let update = UserUpdate {
            first_name: req.first_name.map(Some),
            last_name: req.last_name.map(Some),
            phone: req.phone.map(Some),
            gender: req.gender.map(Some),
            nok_name: req.nok_name.map(Some),
            nok_phone: req.nok_phone.map(Some),
            dob: req.dob.map(Some),
            avatar: req.avatar.map(Some),
            profile_completed: Some(true),
        };

        self.repo
            .update_user(user_id, update)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))
    }

    // ---------- Verify Reset Token ----------
    pub async fn verify_reset_token(&self, token: &str) -> Result<Uuid, AppError> {
        let user_id = self
            .repo
            .verify_reset_token(token)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::BadRequest("Invalid or expired token".to_string()))?;

        Ok(user_id)
    }

    // ---------- Reset Password ----------
    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<(), AppError> {
        // Verify token is valid before hashing password
        let _user_id = self.verify_reset_token(token).await?;

        // Hash new password
        let new_hash = self.hash_password(new_password)?;

        // Update password and delete token
        self.repo
            .reset_password(token, &new_hash)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(())
    }

    fn hash_password(&self, password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::InternalError(format!("Failed to hash password: {}", e)))?
            .to_string();

        Ok(password_hash)
    }

    fn verify_password(&self, password: &str, password_hash: &str) -> Result<(), AppError> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| AppError::InternalError(format!("Invalid password hash: {}", e)))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized("Invalid credentials".to_string()))
    }

    fn generate_token(&self, user: &User) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + chrono::Duration::hours(self.jwt_config.expiration_hours);

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: user.role.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_config.secret.as_bytes()),
        )
        .map_err(|e| AppError::InternalError(e.to_string()))
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, AppError> {
        self.repo
            .find_by_email(email)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, AppError> {
        self.repo
            .find_by_id(user_id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    // ---------- Admin Methods ----------

    pub async fn toggle_user_suspension(
        &self,
        user_id: Uuid,
        suspend: bool,
    ) -> Result<User, AppError> {
        self.repo
            .toggle_user_suspension(user_id, suspend)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<(), AppError> {
        self.repo
            .delete_user(user_id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))
    }

    pub async fn get_all_users(
        &self,
        page: i64,
        limit: i64,
        role_filter: Option<crate::model::UserRole>,
        search_query: Option<String>,
    ) -> Result<(Vec<User>, i64), AppError> {
        self.repo
            .get_all_users(page, limit, role_filter, search_query)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))
    }

    pub async fn get_user_stats(&self) -> Result<(i64, i64, i64), AppError> {
        self.repo
            .get_user_stats()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))
    }

    // ---------- Social Login ----------

    pub async fn social_login(
        &self,
        provider: &str,
        token: &str,
    ) -> Result<AuthResponse, AppError> {
        let (email, first_name, last_name, avatar) = match provider {
            "google" => self.verify_google_token(token).await?,
            "apple" => {
                return Err(AppError::BadRequest(
                    "Apple login not implemented yet".to_string(),
                ));
            }
            _ => return Err(AppError::BadRequest("Invalid provider".to_string())),
        };

        // Find or Create User
        let user = match self
            .repo
            .find_by_email(&email)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
        {
            Some(u) => u,
            None => {
                // Create new user with random password
                let password = Uuid::new_v4().to_string(); // Random password
                let password_hash = self.hash_password(&password)?;

                let mut user = self
                    .repo
                    .create_user(&email, &password_hash, crate::model::UserRole::User)
                    .await
                    .map_err(|e| AppError::InternalError(e.to_string()))?;

                // Update profile with name and avatar
                let update = UserUpdate {
                    first_name: Some(Some(first_name)),
                    last_name: Some(Some(last_name)),
                    avatar: Some(Some(avatar)),
                    phone: None,
                    gender: None,
                    nok_name: None,
                    nok_phone: None,
                    dob: None,
                    profile_completed: Some(true),
                };

                user = self
                    .repo
                    .update_user(user.id, update)
                    .await
                    .map_err(|e| AppError::InternalError(e.to_string()))?;

                // Mark email as verified for social login
                self.repo
                    .mark_user_as_verified(user.id)
                    .await
                    .map_err(|e| AppError::InternalError(e.to_string()))?;

                user
            }
        };

        if user.suspended {
            return Err(AppError::Unauthorized("User is suspended".to_string()));
        }

        let token = self.generate_token(&user)?;
        let refresh_token = self.generate_refresh_token(&user).await?;

        Ok(AuthResponse {
            token,
            refresh_token,
            user: UserInfo {
                id: user.id,
                email: user.email,
                phone: user.phone,
                first_name: user.first_name,
                last_name: user.last_name,
                role: user.role,
                avatar: user.avatar,
                gender: user.gender,
                dob: user.dob,
                nok_name: user.nok_name,
                nok_phone: user.nok_phone,
                email_verified: user.email_verified,
                profile_completed: user.profile_completed,
                profile: user.profile.clone(),
            },
        })
    }

    async fn verify_google_token(
        &self,
        token: &str,
    ) -> Result<(String, String, String, String), AppError> {
        let client = reqwest::Client::new();
        let resp = client
            .get("https://oauth2.googleapis.com/tokeninfo")
            .query(&[("id_token", token)])
            .send()
            .await
            .map_err(|e| {
                AppError::InternalError(format!("Failed to verify Google token: {}", e))
            })?;

        if !resp.status().is_success() {
            return Err(AppError::Unauthorized("Invalid Google token".to_string()));
        }

        let claims: serde_json::Value = resp.json().await.map_err(|e| {
            AppError::InternalError(format!("Failed to parse Google response: {}", e))
        })?;

        let email = claims["email"].as_str().unwrap_or("").to_string();
        let first_name = claims["given_name"].as_str().unwrap_or("").to_string();
        let last_name = claims["family_name"].as_str().unwrap_or("").to_string();
        let avatar = claims["picture"].as_str().unwrap_or("").to_string();

        if email.is_empty() {
            return Err(AppError::Unauthorized(
                "Invalid Google token: missing email".to_string(),
            ));
        }

        Ok((email, first_name, last_name, avatar))
    }
}

pub struct RatingService {
    repo: RatingRepository,
}

impl RatingService {
    pub fn new(repo: RatingRepository) -> Self {
        Self { repo }
    }

    pub async fn create_rating(
        &self,
        user_id: Uuid,
        rater_id: Uuid,
        req: CreateRatingRequest,
    ) -> Result<Rating, AppError> {
        if req.rating < 1.0 || req.rating > 5.0 {
            return Err(AppError::BadRequest(
                "Rating must be between 1 and 5".to_string(),
            ));
        }

        let new_rating = NewRating {
            user_id,
            rater_id,
            rating: req.rating,
            comment: req.comment,
        };

        self.repo
            .create_rating(new_rating)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))
    }

    pub async fn get_user_ratings(&self, user_id: Uuid) -> Result<Vec<Rating>, AppError> {
        self.repo
            .get_user_ratings(user_id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))
    }

    pub async fn get_user_average_rating(&self, user_id: Uuid) -> Result<Option<f64>, AppError> {
        self.repo
            .get_user_average_rating(user_id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))
    }
}
