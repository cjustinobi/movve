use crate::{
    model::{User, UserRole},
    schema::{email_verification_tokens, password_resets, refresh_tokens, users},
};
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::result::Error as DieselError;
use uuid::Uuid;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub id: Uuid,
    pub email: &'a str,
    pub password_hash: &'a str,
    pub role: UserRole, // UserRole enum (it implements Copy/Clone)
    pub email_verified: bool,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserDb {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password_hash: String,
    pub role: UserRole,
    pub email_verified: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<UserDb> for User {
    fn from(user_db: UserDb) -> Self {
        User {
            id: user_db.id,
            email: user_db.email,
            first_name: user_db.first_name,
            last_name: user_db.last_name,
            password_hash: user_db.password_hash,
            role: user_db.role,
            email_verified: user_db.email_verified,
            created_at: user_db.created_at,
            updated_at: user_db.updated_at,
        }
    }
}

#[derive(Queryable, Insertable, Associations, Identifiable, Debug)]
#[diesel(table_name = password_resets)]
#[diesel(belongs_to(UserDb, foreign_key = user_id))]
pub struct PasswordResetDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: NaiveDateTime,
    pub used: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Insertable, Associations, Identifiable, Debug)]
#[diesel(table_name = refresh_tokens)]
#[diesel(belongs_to(UserDb, foreign_key = user_id))]
pub struct RefreshTokenDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: NaiveDateTime,
    pub revoked: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Insertable, Associations, Identifiable, Debug)]
#[diesel(table_name = email_verification_tokens)]
#[diesel(belongs_to(UserDb, foreign_key = user_id))]
pub struct EmailVerificationTokenDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub expires_at: NaiveDateTime,
    pub used: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = refresh_tokens)]
pub struct NewRefreshTokenDb<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: &'a str,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = email_verification_tokens)]
pub struct NewEmailVerificationTokenDb<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: &'a str,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = password_resets)]
pub struct NewPasswordResetDb<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: &'a str,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Clone)]
pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        role: UserRole,
    ) -> Result<User, DbError> {
        let pool = self.pool.clone();
        let email = email.to_string();
        let password_hash = password_hash.to_string();
        // let role_str = role.to_string();

        let user = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;

            let new_user = NewUser {
                id: Uuid::new_v4(),
                email: &email,
                password_hash: &password_hash,
                role,
                email_verified: false,
            };

            let user_db: UserDb = diesel::insert_into(users::table)
                .values(&new_user)
                .returning(UserDb::as_returning())
                .get_result(&mut conn)?;

            Ok::<User, DbError>(user_db.into())
        })
        .await??;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, DbError> {
        let pool = self.pool.clone();
        let email = email.to_string();

        let user = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;

            let user_db = users::table
                .filter(users::email.eq(email))
                .select(UserDb::as_select())
                .first::<UserDb>(&mut conn)
                .optional()?;

            Ok::<Option<User>, DbError>(user_db.map(Into::into))
        })
        .await??;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DbError> {
        let pool = self.pool.clone();

        let user = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;

            let user_db = users::table
                .filter(users::id.eq(id))
                .select(UserDb::as_select())
                .first::<UserDb>(&mut conn)
                .optional()?;

            Ok::<Option<User>, DbError>(user_db.map(Into::into))
        })
        .await??;

        Ok(user)
    }

    // ---------- Forgot Password ----------
    pub async fn create_password_reset(
        &self,
        user_id: Uuid,
        token: &str,
    ) -> Result<String, DbError> {
        let pool = self.pool.clone();
        let token = token.to_string();
        let expires_at = Utc::now() + Duration::minutes(30);
        let created_at = Utc::now().naive_utc();

        let token_clone = token.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            let new_reset = NewPasswordResetDb {
                id: Uuid::new_v4(),
                user_id,
                token: &token_clone,
                expires_at: expires_at.naive_utc(),
                created_at,
            };

            diesel::insert_into(password_resets::table)
                .values(&new_reset)
                .execute(&mut conn)?;

            Ok::<(), DbError>(())
        })
        .await??;

        Ok(token)
    }

    // ---------- Refresh Tokens ----------
    pub async fn create_refresh_token(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: NaiveDateTime,
    ) -> Result<(), DbError> {
        let pool = self.pool.clone();
        let token = token.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            let new_token = NewRefreshTokenDb {
                id: Uuid::new_v4(),
                user_id,
                token: &token,
                expires_at,
                created_at: Utc::now().naive_utc(),
            };

            diesel::insert_into(refresh_tokens::table)
                .values(&new_token)
                .execute(&mut conn)?;

            Ok::<(), DbError>(())
        })
        .await??;

        Ok(())
    }

    pub async fn find_refresh_token(&self, token: &str) -> Result<Option<RefreshTokenDb>, DbError> {
        let pool = self.pool.clone();
        let token = token.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| Box::new(e) as DbError)?;
            refresh_tokens::table
                .filter(refresh_tokens::token.eq(token))
                .filter(refresh_tokens::revoked.eq(false))
                .filter(refresh_tokens::expires_at.gt(Utc::now().naive_utc()))
                .first::<RefreshTokenDb>(&mut conn)
                .optional()
                .map_err(|e| Box::new(e) as DbError)
        })
        .await??;

        Ok(result)
    }

    pub async fn revoke_refresh_token(&self, token: &str) -> Result<(), DbError> {
        let pool = self.pool.clone();
        let token = token.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            diesel::update(refresh_tokens::table.filter(refresh_tokens::token.eq(token)))
                .set(refresh_tokens::revoked.eq(true))
                .execute(&mut conn)?;
            Ok::<(), DbError>(())
        })
        .await??;

        Ok(())
    }

    // ---------- Email Verification ----------
    pub async fn create_verification_code(
        &self,
        user_id: Uuid,
        code: &str,
        expires_at: NaiveDateTime,
    ) -> Result<(), DbError> {
        let pool = self.pool.clone();
        let code = code.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| Box::new(e) as DbError)?;
            let new_token = NewEmailVerificationTokenDb {
                id: Uuid::new_v4(),
                user_id,
                code: &code,
                expires_at,
                created_at: Utc::now().naive_utc(),
            };

            diesel::insert_into(email_verification_tokens::table)
                .values(&new_token)
                .on_conflict(email_verification_tokens::user_id)
                .do_update()
                .set((
                    email_verification_tokens::code.eq(&code),
                    email_verification_tokens::expires_at.eq(expires_at),
                    email_verification_tokens::used.eq(false),
                    email_verification_tokens::created_at.eq(Utc::now().naive_utc()),
                ))
                .execute(&mut conn)
                .map_err(|e| Box::new(e) as DbError)?;

            Ok::<(), DbError>(())
        })
        .await??;

        Ok(())
    }

    pub async fn find_verification_code(
        &self,
        user_id: Uuid,
        code: &str,
    ) -> Result<Option<EmailVerificationTokenDb>, DbError> {
        let pool = self.pool.clone();
        let code = code.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| Box::new(e) as DbError)?;
            email_verification_tokens::table
                .filter(email_verification_tokens::user_id.eq(user_id))
                .filter(email_verification_tokens::code.eq(code))
                .filter(email_verification_tokens::used.eq(false))
                .filter(email_verification_tokens::expires_at.gt(Utc::now().naive_utc()))
                .first::<EmailVerificationTokenDb>(&mut conn)
                .optional()
                .map_err(|e| Box::new(e) as DbError)
        })
        .await??;

        Ok(result)
    }

    pub async fn mark_verification_code_as_used(&self, id: Uuid) -> Result<(), DbError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| Box::new(e) as DbError)?;
            diesel::update(
                email_verification_tokens::table.filter(email_verification_tokens::id.eq(id)),
            )
            .set(email_verification_tokens::used.eq(true))
            .execute(&mut conn)
            .map_err(|e| Box::new(e) as DbError)?;
            Ok::<(), DbError>(())
        })
        .await??;

        Ok(())
    }

    pub async fn mark_user_as_verified(&self, user_id: Uuid) -> Result<(), DbError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| Box::new(e) as DbError)?;
            diesel::update(users::table.filter(users::id.eq(user_id)))
                .set(users::email_verified.eq(true))
                .execute(&mut conn)
                .map_err(|e| Box::new(e) as DbError)?;
            Ok::<(), DbError>(())
        })
        .await??;

        Ok(())
    }

    // ---------- Verify Token ----------
    pub async fn verify_reset_token(&self, token: &str) -> Result<Option<Uuid>, DbError> {
        let pool = self.pool.clone();
        let token_str = token.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            use crate::schema::password_resets::dsl::*;
            let now = Utc::now().naive_utc();

            let reset = password_resets
                .filter(token.eq(&token_str))
                .filter(expires_at.gt(now))
                .first::<PasswordResetDb>(&mut conn)
                .optional()?;

            Ok::<Option<Uuid>, DbError>(reset.map(|r| r.user_id))
        })
        .await??;

        Ok(result)
    }

    // ---------- Reset Password ----------
    pub async fn reset_password(&self, token: &str, new_hash: &str) -> Result<(), DbError> {
        let pool = self.pool.clone();
        let token = token.to_string();
        let new_hash = new_hash.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            use crate::schema::{password_resets::dsl as pr, users::dsl as u};

            // Find valid token
            let now = Utc::now().naive_utc();
            let reset = pr::password_resets
                .filter(pr::token.eq(&token))
                .filter(pr::expires_at.gt(now))
                .first::<PasswordResetDb>(&mut conn)
                .optional()?;

            if let Some(reset) = reset {
                diesel::update(u::users.filter(u::id.eq(reset.user_id)))
                    .set(u::password_hash.eq(new_hash))
                    .execute(&mut conn)?;

                // Delete used token
                diesel::delete(pr::password_resets.filter(pr::id.eq(reset.id)))
                    .execute(&mut conn)?;
            } else {
                return Err(Box::new(DieselError::NotFound) as DbError);
            }

            Ok::<(), DbError>(())
        })
        .await??;

        Ok(())
    }

    pub async fn update_password(&self, user_id: Uuid, new_hash: &str) -> Result<(), DbError> {
        let pool = self.pool.clone();
        let new_hash = new_hash.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            use crate::schema::users::dsl::*;

            diesel::update(users.filter(id.eq(user_id)))
                .set(password_hash.eq(new_hash))
                .execute(&mut conn)?;

            Ok::<(), DbError>(())
        })
        .await??;

        Ok(())
    }
}

#[derive(Clone)]
pub struct PasswordResetRepository {
    pool: DbPool,
}

impl PasswordResetRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create_reset_token(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: NaiveDateTime,
    ) -> Result<(), DbError> {
        let pool = self.pool.clone();
        let token = token.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            let new_reset = NewPasswordResetDb {
                id: Uuid::new_v4(),
                user_id,
                token: &token,
                expires_at,
                created_at: chrono::Utc::now().naive_utc(),
            };
            diesel::insert_into(password_resets::table)
                .values(&new_reset)
                .execute(&mut conn)?;
            Ok::<(), DbError>(())
        })
        .await??;

        Ok(())
    }

    pub async fn find_valid_token(&self, token: &str) -> Result<Option<PasswordResetDb>, DbError> {
        let pool = self.pool.clone();
        let token_str = token.to_string();

        let now = chrono::Utc::now().naive_utc();

        let reset = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            use crate::schema::password_resets::dsl::*;
            let record = password_resets
                .filter(token.eq(&token_str))
                .filter(expires_at.gt(now))
                .first::<PasswordResetDb>(&mut conn)
                .optional()?;
            Ok::<Option<PasswordResetDb>, DbError>(record)
        })
        .await??;

        Ok(reset)
    }

    pub async fn mark_token_as_used(&self, token: &str) -> Result<(), DbError> {
        let pool = self.pool.clone();
        let token_str = token.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            use crate::schema::password_resets::dsl::*;
            diesel::update(password_resets.filter(token.eq(&token_str)))
                .set(used.eq(true))
                .execute(&mut conn)?;
            Ok::<(), DbError>(())
        })
        .await??;

        Ok(())
    }

    pub async fn delete_token(&self, reset_id: Uuid) -> Result<(), DbError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            use crate::schema::password_resets::dsl::*;
            diesel::delete(password_resets.filter(id.eq(reset_id))).execute(&mut conn)?;
            Ok::<(), DbError>(())
        })
        .await??;

        Ok(())
    }
}
