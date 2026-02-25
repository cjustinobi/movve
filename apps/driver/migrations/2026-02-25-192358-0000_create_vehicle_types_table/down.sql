CREATE TYPE vehicle_type AS ENUM ('sedan', 'suv', 'van', 'motorcycle');

ALTER TABLE drivers
    ALTER COLUMN vehicle_type TYPE vehicle_type USING
    CASE vehicle_type
        WHEN 'sedan' THEN 'sedan'::vehicle_type
        WHEN 'suv' THEN 'suv'::vehicle_type
        WHEN 'van' THEN 'van'::vehicle_type
        WHEN 'motorcycle' THEN 'motorcycle'::vehicle_type
        -- IF there are customized dynamic types they will fail to cast implicitly
        ELSE 'sedan'::vehicle_type
    END;

DROP TABLE vehicle_types;
