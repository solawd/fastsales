# FastSales API

Simple Axum-based CRUD API with JWT authentication, SQLite storage, and Swagger UI.

## Requirements

- Rust toolchain (stable)
- SQLite (via file storage)

## Setup

1. Clone the repository and enter the project directory.
2. Optional environment variables:
   - `DATABASE_URL` (default: local `fastsales.db`)
   - `JWT_SECRET` (default: `dev-secret`)
   - `PASSWORD_PEPPER` (default: empty string)

## Run

```bash
cargo run
```

The server starts on `http://127.0.0.1:3000`.

## Authentication

Default staff user seeded by migration:

- username: `admin`
- password: `password123`

Login:

```bash
curl -X POST http://127.0.0.1:3000/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"password123"}'
```

Use the returned token:

```bash
curl http://127.0.0.1:3000/products \
  -H 'Authorization: Bearer <token>'
```

## Swagger / OpenAPI

- Swagger UI: `http://127.0.0.1:3000/swagger-ui/`
- OpenAPI JSON: `http://127.0.0.1:3000/api-doc/openapi.json`

## Database

Migrations are applied on startup via `sqlx::migrate!()`.
The default database file is `fastsales.db` in the project root, unless `DATABASE_URL` is set.

## Environment-specific configuration

Recommended environment variables by environment:

### Development

```
DATABASE_URL=sqlite://./fastsales.db
JWT_SECRET=dev-secret
PASSWORD_PEPPER=
```

### Production

```
DATABASE_URL=sqlite:///var/lib/fastsales/fastsales.db
JWT_SECRET=change-me-prod-secret
PASSWORD_PEPPER=change-me-prod-pepper
```

Notes:
- Ensure the directory for `DATABASE_URL` exists and is writable by the app.
- Changing `PASSWORD_PEPPER` invalidates existing password hashes.
