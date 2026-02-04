use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::Utc;
use common::{AppError, JwtConfig};
use jsonwebtoken::{EncodingKey, Header, encode};
use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::{
    model::{
        AuthResponse, Claims, LoginRequest, RegisterRequest, UpdateProfileRequest, User, UserInfo,
    },
    repository::{UserRepository, UserUpdate},
};

pub struct AuthService {
    repo: UserRepository,
    pub jwt_config: JwtConfig,
}

impl AuthService {
    pub fn new(repo: UserRepository, jwt_config: JwtConfig) -> Self {
        Self { repo, jwt_config }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, AppError> {
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

        let token = self.generate_token(&user)?;
        let refresh_token = self.generate_refresh_token(&user).await?;

        Ok(AuthResponse {
            token,
            refresh_token,
            user: UserInfo {
                id: user.id,
                email: user.email,
                role: user.role,
                avatar: user.avatar,
                email_verified: user.email_verified,
            },
        })
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
                role: user.role,
                avatar: user.avatar,
                email_verified: user.email_verified,
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
                role: user.role,
                avatar: user.avatar,
                email_verified: user.email_verified,
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

    pub async fn verify_email(&self, user_id: Uuid, code: &str) -> Result<(), AppError> {
        // 1. Check if user is already verified
        let user = self
            .repo
            .find_by_id(user_id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if user.email_verified {
            return Err(AppError::BadRequest("Email already verified".to_string()));
        }

        // 2. Find valid verification code
        let token = self
            .repo
            .find_verification_code(user_id, code)
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
            .mark_user_as_verified(user_id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(())
    }

    pub async fn update_profile(
        &self,
        user_id: Uuid,
        req: UpdateProfileRequest,
    ) -> Result<User, AppError> {
        let update = UserUpdate {
            first_name: req.first_name.map(Some),
            last_name: req.last_name.map(Some),
            phone: req.phone.map(Some),
            gender: req.gender.map(Some),
            nok_name: req.nok_name.map(Some),
            nok_phone: req.nok_phone.map(Some),
            dob: req.dob.map(Some),
            avatar: req.avatar.map(Some),
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
}
