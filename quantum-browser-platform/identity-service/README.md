Identity Service (scaffold)

This service provides authentication and user management for QuantumEnergyOS.

Tech choices (initial):
- Rust + Axum
- PostgreSQL (sqlx)
- JWT (jsonwebtoken)
- Password hashing: argon2

Quickstart (dev):
- Required environment variables:
  - DATABASE_URL: Postgres connection URL (e.g., postgres://user:pass@localhost:5432/dbname)
  - JWT_SECRET: Strong secret used to sign JWTs (must be set; service will fail to start without it)
- Load env in development via a .env file and dotenvy
- Run migrations in sql/migrations/* against your Postgres DB
- cargo run --bin identity_service

Notes on testing:
- Integration tests use testcontainers and require Docker available on the runner.
- CI workflow runs the integration tests in a dedicated job.

Planned endpoints (initial):
- GET /health
- POST /auth/register {email, password}
- POST /auth/login {email, password} -> { token }
- GET /users/me (requires auth)

Next steps:
- Add middleware for JWT validation
- Add roles and permissions
- Add 2FA support (optional)
- Add tests and CI
