CREATE TYPE driver_status AS ENUM ('offline', 'online', 'busy');
CREATE TYPE vehicle_type AS ENUM ('sedan', 'suv', 'van', 'motorcycle');

CREATE TABLE drivers (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    phone VARCHAR(20) NOT NULL,
    license_number VARCHAR(50) NOT NULL UNIQUE,
    vehicle_type vehicle_type NOT NULL,
    vehicle_plate VARCHAR(20) NOT NULL,
    vehicle_model VARCHAR(100) NOT NULL,
    vehicle_year INTEGER NOT NULL,
    status driver_status NOT NULL DEFAULT 'offline',
    rating DECIMAL(3, 2) DEFAULT 5.0,
    total_rides INTEGER DEFAULT 0,
    current_latitude DECIMAL(10, 8),
    current_longitude DECIMAL(11, 8),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_drivers_user_id ON drivers(user_id);
CREATE INDEX idx_drivers_status ON drivers(status);
CREATE INDEX idx_drivers_location ON drivers(current_latitude, current_longitude) WHERE status = 'online';

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_drivers_updated_at BEFORE UPDATE ON drivers
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();