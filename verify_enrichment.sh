#!/bin/bash
set -e

echo "Registering user..."
REGISTER_RESP=$(curl -s -X POST http://localhost:8001/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test_enrich_driver_FINAL@example.com",
    "password": "password123",
    "role": "driver"
  }')

echo "Register Response: $REGISTER_RESP"

# Extract ID and Token using simple grep/cut
USER_ID=$(echo $REGISTER_RESP | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
TOKEN=$(echo $REGISTER_RESP | grep -o '"token":"[^"]*"' | head -1 | cut -d'"' -f4)

echo "User ID: $USER_ID"
# echo "Token: $TOKEN"

if [ -z "$USER_ID" ]; then
  echo "Failed to extract User ID. Registration failed."
  exit 1
fi

echo "Creating Driver Profile..."
DRIVER_RESP=$(curl -s -X POST http://localhost:8002/api/driver/drivers \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "'"$USER_ID"'",
    "license_number": "LIC-FINAL",
    "driver_license_image": "http://example.com/lic.jpg",
    "vehicle_image": "http://example.com/car.jpg",
    "vehicle_type": "sedan",
    "vehicle_colour": "black",
    "vehicle_plate": "ABC-FINAL",
    "vehicle_model": "Toyota Camry",
    "vehicle_year": 2020,
    "status": "offline"
  }')

echo "Create Driver Response: $DRIVER_RESP"

if [[ $DRIVER_RESP == *"error"* ]]; then
    echo "Driver creation failed!"
fi

echo "Fetching Me..."
ME_RESP=$(curl -s -X GET http://localhost:8001/api/auth/me \
  -H "Authorization: Bearer $TOKEN")

echo "Me Response: $ME_RESP"

if [[ "$ME_RESP" == *'"profile":{'* ]]; then
  echo "SUCCESS: Profile found in Me response"
else
  echo "FAILURE: Profile NOT found in Me response"
fi
