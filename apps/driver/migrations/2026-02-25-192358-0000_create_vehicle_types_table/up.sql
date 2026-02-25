CREATE TABLE vehicle_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    base_price FLOAT8 NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

INSERT INTO vehicle_types (name, display_name, description, base_price) VALUES
    ('sedan', 'Sedan', 'Comfortable car for up to 4 passengers', 500.0),
    ('suv', 'SUV', 'Spacious vehicle for larger groups or luggage', 800.0),
    ('van', 'Van', 'Big van for moving people or goods', 1200.0),
    ('motorcycle', 'Motorcycle', 'Fast and affordable ride for one passenger', 300.0);

ALTER TABLE drivers ALTER COLUMN vehicle_type TYPE VARCHAR(50) USING vehicle_type::text;
DROP TYPE vehicle_type;
