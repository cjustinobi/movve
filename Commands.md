# Important commands Usage Examples


## Development

### Show all available commands
- `cargo make help`

### Initial setup
- `cargo make setup`
### or
`./scripts/setup.sh`

### Run all services in tmux (recommended)
- `cargo make run-all`

### Stop all services
- `cargo make stop-all`

### Watch individual services (hot reload)
- `cargo make watch-auth      # Terminal 1`
- `cargo make watch-gateway   # Terminal 2`
- `cargo make watch-driver    # Terminal 3`

## Building

### Build everything
- `cargo make build`

### Build release version
- `cargo make build-release`

### Build individual services
- `cargo make build-auth`
- `cargo make build-gateway`
- `cargo make build-driver`

## Database

### Create database
- `cargo make db-create`

### Run migrations
- `cargo make db-migrate`

### Reset database
- `cargo make db-reset`

### Complete setup
- `cargo make db-setup`

## Testing

### Run all tests
- `cargo make test`

### Test specific service
- `cargo make test-auth`
- `cargo make test-gateway`
- `cargo make test-driver`

## Linting & Formating

### Format code
- `cargo make fmt`

### Check formatting
- `cargo make fmt-check`

### Run clippy
- `cargo make clippy`

### Run all linting
- `cargo make lint`

### Pre-commit checks
- `cargo make pre-commit`

## Docker

### Build Docker images
- `cargo make docker-build`

### Start services with docker-compose
- `cargo make docker-up`

### Stop services
- `cargo make docker-down`

### View logs
- `cargo make docker-logs`

## CI/CD

### Run CI checks (for CI pipeline)
- `cargo make ci`

### Run everything
- `cargo make all`

## Utilities

### Clean build artifacts
- `cargo make clean`

### Check for outdated dependencies
- `cargo make deps`

### Security audit
- `cargo make audit`

### Show dependency tree
- `cargo make tree`