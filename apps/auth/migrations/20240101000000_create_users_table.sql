CREATE TYPE user_role AS ENUM ('admin', 'driver', 'vendor', 'dispatcher', 'user');
CREATE TYPE gender AS ENUM ('male', 'female');

CREATE TABLE users (
    id UUID PRIMARY KEY,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    email VARCHAR(100) UNIQUE NOT NULL,
    phone VARCHAR(100) UNIQUE,
    gender gender NOT NULL,
    nok_name VARCHAR(100),
    nok_phone VARCHAR(100),
    dob DATE,
    password_hash VARCHAR(255) NOT NULL,
    role user_role NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);