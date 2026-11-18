#!/bin/sh

# 01-init-security.sh
# Creates the least-privilege application user and grants it read access to the service schema.
# Usage: run once by the PostgreSQL image entrypoint on first container start, never invoked standalone.
# Arguments: none. Reads CONGESTION_TAX_APP_USER, CONGESTION_TAX_APP_PASSWORD, POSTGRES_USER, and POSTGRES_DB from the
# environment.

: "${CONGESTION_TAX_APP_USER:?missing}"
: "${CONGESTION_TAX_APP_PASSWORD:?missing}"

psql -v ON_ERROR_STOP=1 --username "${POSTGRES_USER}" --dbname "${POSTGRES_DB}" <<EOSQL
CREATE ROLE "${CONGESTION_TAX_APP_USER}" LOGIN PASSWORD '${CONGESTION_TAX_APP_PASSWORD}';
GRANT CONNECT ON DATABASE "${POSTGRES_DB}" TO "${CONGESTION_TAX_APP_USER}";
GRANT USAGE ON SCHEMA public TO "${CONGESTION_TAX_APP_USER}";
ALTER DEFAULT PRIVILEGES FOR ROLE "${POSTGRES_USER}" IN SCHEMA public
    GRANT SELECT ON TABLES TO "${CONGESTION_TAX_APP_USER}";
EOSQL
