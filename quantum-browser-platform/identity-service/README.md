Identity Service (scaffold)

This service provides authentication and user management for QuantumEnergyOS.

Tech choices (initial):
- Rust + Axum
- PostgreSQL (sqlx)
- JWT (jsonwebtoken)
- Password hashing: argon2

Quickstart (dev):
- Set DATABASE_URL and JWT_SECRET in env or .env file
- Run migrations in sql/migrations/* against your Postgres DB
- cargo run --bin identity_service

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
