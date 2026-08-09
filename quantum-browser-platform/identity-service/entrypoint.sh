#!/usr/bin/env bash
set -euo pipefail

# Entrypoint for identity_service Docker image.
# Waits for DATABASE_URL to be available, runs SQL migrations in ./sql/migrations,
# then execs the identity_service binary.

if [ -z "${DATABASE_URL:-}" ]; then
  echo "DATABASE_URL is not set. Exiting."
  exit 1
fi

echo "Waiting for database to be ready..."

# Wait for postgres to accept connections using psql
until psql "$DATABASE_URL" -c '\q' >/dev/null 2>&1; do
  echo "Waiting for postgres..."
  sleep 1
done

echo "Database is up. Running migrations..."

MIG_DIR="./sql/migrations"
if [ -d "$MIG_DIR" ]; then
  for f in "$MIG_DIR"/*.sql; do
    [ -e "$f" ] || continue
    echo "Applying migration: $f"
    psql "$DATABASE_URL" -f "$f"
  done
else
  echo "No migrations directory found at $MIG_DIR"
fi

echo "Migrations applied. Launching identity_service..."

# Exec the service binary (forward args if any)
exec /usr/local/bin/identity_service "$@"
