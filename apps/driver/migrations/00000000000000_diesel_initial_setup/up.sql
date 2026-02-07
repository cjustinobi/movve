CREATE TYPE driver_status AS ENUM ('offline', 'online', 'busy');
CREATE TYPE vehicle_type AS ENUM ('sedan', 'suv', 'van', 'motorcycle');
CREATE TYPE vehicle_colour AS ENUM ('red', 'blue', 'green', 'gray', 'black', 'white', 'silver', 'yellow');

CREATE TABLE drivers (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    license_number VARCHAR(50) NOT NULL UNIQUE,
    driver_license_image VARCHAR(255) NOT NULL,
    insurance_number VARCHAR(50) UNIQUE,
    insurance_image VARCHAR(255),
    vehicle_image VARCHAR(255) NOT NULL,
    vehicle_type vehicle_type NOT NULL,
    vehicle_colour vehicle_colour NOT NULL,
    vehicle_plate VARCHAR(20) NOT NULL UNIQUE,
    vehicle_model VARCHAR(100) NOT NULL,
    vehicle_year INTEGER NOT NULL,
    status driver_status NOT NULL DEFAULT 'offline',
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    suspended BOOLEAN NOT NULL DEFAULT FALSE,
    vehicle_verification_completed BOOLEAN NOT NULL DEFAULT FALSE,
    vehicle_capacity INTEGER NOT NULL DEFAULT 4,
    driver_license_verified BOOLEAN NOT NULL DEFAULT FALSE,
    insurance_verified BOOLEAN NOT NULL DEFAULT FALSE,
    vehicle_image_verified BOOLEAN NOT NULL DEFAULT FALSE,
    rating DECIMAL(3, 2) DEFAULT 5.0,
    total_rides INTEGER DEFAULT 0,
    current_latitude DECIMAL(10, 8),
    current_longitude DECIMAL(11, 8),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_drivers_user_id ON drivers(user_id);
CREATE INDEX idx_drivers_status ON drivers(status);
CREATE INDEX idx_drivers_location ON drivers(current_latitude, current_longitude) WHERE status = 'online';



-- Trigger

CREATE OR REPLACE FUNCTION diesel_manage_updated_at(_tbl regclass) RETURNS VOID AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION diesel_set_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
