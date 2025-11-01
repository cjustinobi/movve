// apps/auth/src/repository.rs
use common::{User, UserRole};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use uuid::Uuid;
use chrono::NaiveDateTime;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbError = Box<dyn std::error::Error + Send + Sync>;

use crate::schema::users;

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub id: Uuid,
    pub email: &'a str,
    pub password_hash: &'a str,
    pub role: &'a str,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct UserDb {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<UserDb> for User {
    fn from(user_db: UserDb) -> Self {
        User {
            id: user_db.id,
            email: user_db.email,
            password_hash: user_db.password_hash,
            role: match user_db.role.as_str() {
                "admin" => UserRole::Admin,
                "driver" => UserRole::Driver,
                "dispatcher" => UserRole::Dispatcher,
                "vendor" => UserRole::Vendor,
                "user" => UserRole::User,
                _ => UserRole::User, // default fallback
            },
            created_at: user_db.created_at,
            updated_at: user_db.updated_at,
        }
    }
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
        let role_str = role.to_string();
        
        let user = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            
            let new_user = NewUser {
                id: Uuid::new_v4(),
                email: &email,
                password_hash: &password_hash,
                role: &role_str,
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
}