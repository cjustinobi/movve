use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use common::{AppError, AuthResponse, Claims, JwtConfig, LoginRequest, RegisterRequest, User, UserInfo};
use jsonwebtoken::{encode, EncodingKey, Header};
use uuid::Uuid;

use crate::repository::UserRepository;

pub struct AuthService {
    repo: UserRepository,
    jwt_config: JwtConfig,
}

impl AuthService {
    pub fn new(repo: UserRepository, jwt_config: JwtConfig) -> Self {
        Self { repo, jwt_config }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, AppError> {
        if self.repo.find_by_email(&req.email).await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .is_some()
        {
            return Err(AppError::Conflict("Email already exists".to_string()));
        }

        let password_hash = hash(&req.password, DEFAULT_COST)
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let user = self.repo.create_user(&req.email, &password_hash, req.role).await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let token = self.generate_token(&user)?;

        Ok(AuthResponse {
            token,
            user: UserInfo {
                id: user.id,
                email: user.email,
                role: user.role,
            },
        })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, AppError> {
        let user = self.repo.find_by_email(&req.email).await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

        let valid = verify(&req.password, &user.password_hash)
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if !valid {
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }

        let token = self.generate_token(&user)?;

        Ok(AuthResponse {
            token,
            user: UserInfo {
                id: user.id,
                email: user.email,
                role: user.role,
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
}