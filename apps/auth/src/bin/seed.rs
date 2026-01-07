use diesel::prelude::*;
use uuid::Uuid;
use chrono::Utc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use auth_seed::{
    schema::users,
    model::{UserRole},
};

#[derive(Insertable)]
#[diesel(table_name = users)]
struct NewUser<'a> {
    id: Uuid,
    email: &'a str,
    first_name: Option<&'a str>,
    last_name: Option<&'a str>,
    password_hash: &'a str,
    role: UserRole,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

fn hash_password(password: &str) -> Result<String, Box<dyn std::error::Error>> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut conn =
        PgConnection::establish(&database_url)
            .expect("Failed to connect to database");

    println!("🚀 Starting auth service seeding...");

    // Clear existing users
    diesel::sql_query("DELETE FROM users").execute(&mut conn)?;
    println!("✓ Cleared users table");

    let password_hash = hash_password("password")?;
    let now = Utc::now().naive_utc();

    let users_to_seed = vec![
        NewUser {
            id: Uuid::new_v4(),
            email: "admin@mail.com",
            first_name: Some("System"),
            last_name: Some("Admin"),
            password_hash: &password_hash,
            role: UserRole::Admin,
            created_at: now,
            updated_at: now,
        },
        NewUser {
            id: Uuid::new_v4(),
            email: "vendor@mail.com",
            first_name: Some("Vendor"),
            last_name: Some("User"),
            password_hash: &password_hash,
            role: UserRole::Vendor,
            created_at: now,
            updated_at: now,
        },
        NewUser {
            id: Uuid::new_v4(),
            email: "driver@mail.com",
            first_name: Some("Driver"),
            last_name: Some("User"),
            password_hash: &password_hash,
            role: UserRole::Driver,
            created_at: now,
            updated_at: now,
        },
        NewUser {
            id: Uuid::new_v4(),
            email: "dispatcher@mail.com",
            first_name: Some("Dispatcher"),
            last_name: Some("User"),
            password_hash: &password_hash,
            role: UserRole::Dispatcher,
            created_at: now,
            updated_at: now,
        },
        NewUser {
            id: Uuid::new_v4(),
            email: "user@mail.com",
            first_name: Some("Regular"),
            last_name: Some("User"),
            password_hash: &password_hash,
            role: UserRole::User,
            created_at: now,
            updated_at: now,
        },
    ];

    diesel::insert_into(users::table)
        .values(&users_to_seed)
        .execute(&mut conn)?;

    println!("✓ Seeded users:");

    for user in &users_to_seed {
        println!("  - {} ({:?})", user.email, user.role);
    }

    println!("\nLogin password for all users: password");
    println!("✅ Auth seeding complete");

    Ok(())
}
