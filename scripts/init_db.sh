#!/usr/bin/env bash

set -x
set -eo pipefail

# check if psql is installed, else exit
if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi

# check if sqlx-cli is installed, else exit
if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error sqlx-cli is not installed"
  echo >&2 "Use:"
  echo >&2 " cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres"
  echo >&2 "to install it"
  exit 1
fi

# Check if a custom user has been set, otherwise default to 'postgres'
DB_USER=${POSTGRES_USER:=postgres}
# Check if a custom password is set, else default to 'password'
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
# Check if a custom db name has been set, else default to 'newsletter'
DB_NAME="${POSTGRES_DB:=newsletter}"
# Check if a custom port has been set, otherwise default to 5432
DB_PORT="${POSTGRES_PORT:=5432}"


# ALLOW TO SKTOP DOCKER IF A DOCKERIZED POSTGRES DATABASE IS ALREADY RUNNING
if [[ -z "${SKIP_DOCKER}" ]]
then
  # Launch
  docker run \
  -e POSTGRES_USER=${DB_USER} \
  -e POSTGRES_PASSWORD=${DB_PASSWORD} \
  -e POSTGRES_DB=${DB_NAME} \
  -p "${DB_PORT}":5432 \
  -d postgres \
  postgres -N 1000
fi

# ^ Increased the number of connections for testing purpose

# Keep pinging Postgres until its ready to accept commands
export PGPASSWORD=${DB_PASSWORD}
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  >&2 echo "Postgres is unavailable or sleeping"
  sleep 1
done

>&2 echo "Postgres is running on port: ${DB_PORT}"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated! Ready to use!"