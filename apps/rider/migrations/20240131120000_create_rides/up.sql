CREATE TABLE rides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rider_id UUID NOT NULL,
    driver_id UUID,
    pickup JSONB NOT NULL,
    destination JSONB NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'requested',
    fare DOUBLE PRECISION NOT NULL,
    distance DOUBLE PRECISION NOT NULL DEFAULT 0.0,  -- Distance in meters
    duration DOUBLE PRECISION NOT NULL DEFAULT 0.0,  -- Duration in seconds
    otp VARCHAR(10),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rides_rider_id ON rides(rider_id);
CREATE INDEX idx_rides_driver_id ON rides(driver_id);
CREATE INDEX idx_rides_status ON rides(status);
CREATE INDEX idx_rides_created_at ON rides(created_at DESC);

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_rides_updated_at BEFORE UPDATE ON rides
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add comments for documentation
COMMENT ON TABLE rides IS 'Stores ride requests and their details';
COMMENT ON COLUMN rides.pickup IS 'JSON object containing address, latitude, and longitude';
COMMENT ON COLUMN rides.destination IS 'JSON object containing address, latitude, and longitude';
COMMENT ON COLUMN rides.distance IS 'Actual distance in meters calculated from pickup to destination';
COMMENT ON COLUMN rides.duration IS 'Actual duration in seconds calculated for the route';
COMMENT ON COLUMN rides.status IS 'Ride status: requested, accepted, started, completed, cancelled, paid';
COMMENT ON COLUMN rides.otp IS 'One-time password for ride verification';
