use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use common::{
    AppError, AuthResponse, Claims, JwtConfig, LoginRequest, RegisterRequest, User, UserInfo,
};
use jsonwebtoken::{encode, EncodingKey, Header};

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
        let user = self
            .repo
            .find_by_email(&req.email)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

        self.verify_password(&req.password, &user.password_hash)?;

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
}


// use argon2::{
//     password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
//     Argon2,
// };
// use chrono::Utc;
// use common::{
//     AppError, AuthResponse, Claims, JwtConfig, LoginRequest, RegisterRequest, User, UserInfo,
// };
// use jsonwebtoken::{encode, EncodingKey, Header};

// use crate::repository::UserRepository;

// pub struct AuthService {
//     repo: UserRepository,
//     jwt_config: JwtConfig,
// }

// impl AuthService {
//     pub fn new(repo: UserRepository, jwt_config: JwtConfig) -> Self {
//         Self { repo, jwt_config }
//     }

//     pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, AppError> {
//         if self
//             .repo
//             .find_by_email(&req.email)
//             .await
//             .map_err(|e| AppError::InternalError(e.to_string()))?
//             .is_some()
//         {
//             return Err(AppError::Conflict("Email already exists".to_string()));
//         }

//         let password_hash = self.hash_password(&req.password)?;

//         let user = self
//             .repo
//             .create_user(&req.email, &password_hash, req.role)
//             .await
//             .map_err(|e| AppError::InternalError(e.to_string()))?;

//         let token = self.generate_token(&user)?;

//         Ok(AuthResponse {
//             token,
//             user: UserInfo {
//                 id: user.id,
//                 email: user.email,
//                 role: user.role,
//             },
//         })
//     }

//     pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, AppError> {
//         let user = self
//             .repo
//             .find_by_email(&req.email)
//             .await
//             .map_err(|e| AppError::InternalError(e.to_string()))?
//             .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

//         self.verify_password(&req.password, &user.password_hash)?;

//         let token = self.generate_token(&user)?;

//         Ok(AuthResponse {
//             token,
//             user: UserInfo {
//                 id: user.id,
//                 email: user.email,
//                 role: user.role,
//             },
//         })
//     }

//     pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
//         let claims = jsonwebtoken::decode::<Claims>(
//             token,
//             &jsonwebtoken::DecodingKey::from_secret(self.jwt_config.secret.as_bytes()),
//             &jsonwebtoken::Validation::default(),
//         )
//         .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?
//         .claims;

//         Ok(claims)
//     }

//     fn hash_password(&self, password: &str) -> Result<String, AppError> {
//         let salt = SaltString::generate(&mut OsRng);
//         let argon2 = Argon2::default();

//         let password_hash = argon2
//             .hash_password(password.as_bytes(), &salt)
//             .map_err(|e| AppError::InternalError(format!("Failed to hash password: {}", e)))?
//             .to_string();

//         Ok(password_hash)
//     }

//     fn verify_password(&self, password: &str, password_hash: &str) -> Result<(), AppError> {
//         let parsed_hash = PasswordHash::new(password_hash)
//             .map_err(|e| AppError::InternalError(format!("Invalid password hash: {}", e)))?;

//         Argon2::default()
//             .verify_password(password.as_bytes(), &parsed_hash)
//             .map_err(|_| AppError::Unauthorized("Invalid credentials".to_string()))
//     }

//     fn generate_token(&self, user: &User) -> Result<String, AppError> {
//         let now = Utc::now();
//         let exp = now + chrono::Duration::hours(self.jwt_config.expiration_hours);

//         let claims = Claims {
//             sub: user.id.to_string(),
//             email: user.email.clone(),
//             role: user.role.clone(),
//             exp: exp.timestamp(),
//             iat: now.timestamp(),
//         };

//         encode(
//             &Header::default(),
//             &claims,
//             &EncodingKey::from_secret(self.jwt_config.secret.as_bytes()),
//         )
//         .map_err(|e| AppError::InternalError(e.to_string()))
//     }
// }