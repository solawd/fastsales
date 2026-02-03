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

The project is a workspace with two crates: `backend` and `frontend`.

### Backend (Server)

To run the Axum server (which also serves the frontend assets if built):

```bash
cargo run --bin backend
```

The server starts on `http://127.0.0.1:3000`.

### Frontend (Leptos)

The frontend is a Leptos app that compiles to WASM. To develop with live reloading, run from the **project root**:

1. Install `cargo-leptos`:
   ```bash
   cargo install cargo-leptos
   ```

2. Run watch:
   ```bash
   cargo leptos watch
   ```

3. Or build for production (assets go to `target/site`):
   ```bash
   cargo leptos build --release
   ```


## Authentication

Default staff user seeded by migration:

- username: `admin`
- password: `password123`

Login:

```bash
curl -X POST http://127.0.0.1:3000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"password123"}'
```

Use the returned token:

```bash
curl http://127.0.0.1:3000/api/products \
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
