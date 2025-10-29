# Prerequisite
- Ensure you have latest rust version installed - Run `rustup update stable` to update to the latest stable version



# 1. Clone and setup
git clone [https://github.com/Moove-Buildl/movve_monorepo.git](https://github.com/Moove-Buildl/movve_monorepo.git)

cd to Movve repo

Run `./scripts/setup.sh`

# 2. Start all services

Run `cargo make run-all`

# 3. In another terminal, test the API
```
#Register
curl -X POST http://localhost:8000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123","role":"user"}'

# Login
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'

# Verify token (use token from login response)
curl -X GET http://localhost:8000/api/auth/verify \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIyMjlmZTRkMS1iYThlLTQ0MGYtOWVjYy0wZDNlYjY5NTdhMWMiLCJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJyb2xlIjoiVXNlciIsImV4cCI6MTc2MTcwNjgzNCwiaWF0IjoxNzYxNjIwNDM0fQ.lw5UwUgI9xlN0sReQKsgATyc8b69sgM7Svp46KdRVTc"
  ```

# 4. Stop services
cargo make stop-all