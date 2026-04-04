# FieldTrace Rescue & Supply Chain

Offline-first shelter and warehouse management system.

## Stack

- **Backend**: Axum (Rust)
- **Frontend**: Leptos (Rust/WASM, CSR)
- **Database**: SQLite (embedded, single-file)

## Quick Start

```bash
docker compose up --build
```

The application will be available at **http://localhost:8080**.

- API: `http://localhost:8080/health`
- UI: `http://localhost:8080/`

## Running Tests

```bash
chmod +x run_tests.sh
./run_tests.sh
```

Tests run inside the Docker container. The script handles container startup and health checks automatically.

## Environment Variables

All environment variables are defined inline in `docker-compose.yml`. No `.env` file required.

| Variable | Default | Description |
|---|---|---|
| PORT | 8080 | Server listen port |
| DATABASE_URL | sqlite:///app/storage/app.db | SQLite database path |
| STATIC_DIR | /app/static | Frontend static files directory |
| RUST_LOG | info | Log level filter |
| ENCRYPTION_KEY | (set in compose) | AES-256 encryption key (hex) |

## Test Credentials

No credentials required for Slice 1 (health endpoint only). Auth is added in Slice 2.
