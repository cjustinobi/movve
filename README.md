# 1. Clone and setup
git clone <your-repo>
cd <your-monorepo>
./scripts/setup.sh

# 2. Start all services
cargo make run-all

# 3. In another terminal, test the API
```
curl -X POST http://localhost:8000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123","role":"user"}'
  ```

# 4. Stop services
cargo make stop-all