# GitJobs - Development Tasks
#
# Configuration: Set these environment variables.
#
#   Optional (with defaults):
#     GITJOBS_CONFIG        - Path to config directory (default: $HOME/.config/gitjobs)
#     GITJOBS_DB_HOST       - Database host or unix socket path (default: localhost)
#     GITJOBS_DB_NAME       - Main database name (default: gitjobs)
#     GITJOBS_DB_NAME_TESTS - Test database name (default: gitjobs_tests)
#     GITJOBS_DB_PORT       - Database port (default: 5432)
#     GITJOBS_DB_USER       - Database user (default: postgres)
#     GITJOBS_PG_BIN        - Path to PostgreSQL binaries
#                             (default: /opt/homebrew/opt/postgresql@17/bin)
#
# Please set up tern config files (`tern.conf` and `tern-tests.conf`) in
# `GITJOBS_CONFIG` with connection settings matching these variables.

# Configuration
config_dir := env("GITJOBS_CONFIG", env_var("HOME") / ".config/gitjobs")
db_host := env("GITJOBS_DB_HOST", "localhost")
db_name := env("GITJOBS_DB_NAME", "gitjobs")
db_name_tests := env("GITJOBS_DB_NAME_TESTS", "gitjobs_tests")
db_port := env("GITJOBS_DB_PORT", "5432")
db_server_host_opt := if db_host =~ '^/' { "-k " + db_host } else { "-h " + db_host }
db_user := env("GITJOBS_DB_USER", "postgres")
pg_bin := env("GITJOBS_PG_BIN", "/opt/homebrew/opt/postgresql@17/bin")
pg_conn := "-h " + db_host + " -p " + db_port + " -U " + db_user
source_dir := justfile_directory()

# Helper to run PostgreSQL commands with the configured binary path.
[private]
pg command *args:
    PATH="{{ pg_bin }}:$PATH" {{ command }} {{ args }}

# Database

# Connect to main database.
db-client:
    just pg psql {{ pg_conn }} {{ db_name }}

# Connect to test database.
db-client-tests:
    just pg psql {{ pg_conn }} {{ db_name_tests }}

# Create main database.
db-create:
    just pg createdb {{ pg_conn }} {{ db_name }}

# Create test database with pgtap extension.
db-create-tests:
    just pg createdb {{ pg_conn }} {{ db_name_tests }}
    PATH="{{ pg_bin }}:$PATH" psql {{ pg_conn }} {{ db_name_tests }} -c "CREATE EXTENSION IF NOT EXISTS pgtap"

# Drop main database.
db-drop:
    just pg dropdb {{ pg_conn }} --if-exists --force {{ db_name }}

# Drop test database.
db-drop-tests:
    just pg dropdb {{ pg_conn }} --if-exists --force {{ db_name_tests }}

# Initialize PostgreSQL data directory.
db-init data_dir:
    mkdir -p "{{ data_dir }}"
    just pg initdb -U {{ db_user }} "{{ data_dir }}"

# Load e2e seed data into main database.
db-load-e2e-data:
    just pg psql {{ pg_conn }} {{ db_name }} -f "{{ source_dir }}/database/tests/data/e2e.sql"

# Run migrations on main database.
db-migrate:
    cd "{{ source_dir }}/database/migrations" && TERN_CONF="{{ config_dir }}/tern.conf" ./migrate.sh

# Run migrations on test database.
db-migrate-tests:
    cd "{{ source_dir }}/database/migrations" && TERN_CONF="{{ config_dir }}/tern-tests.conf" ./migrate.sh

# Drop, create, and migrate main database.
db-recreate: db-drop db-create db-migrate

# Drop, create, and migrate test database.
db-recreate-tests: db-drop-tests db-create-tests db-migrate-tests

# Start PostgreSQL server.
db-server data_dir:
    just pg postgres -D "{{ data_dir }}" -p {{ db_port }} {{ db_server_host_opt }}

# Server

# Run the server using cargo run (builds if needed).
server:
    cargo run -p gitjobs-server -- --config-file "{{ config_dir }}/server.yml"

# Build the server binary.
server-build:
    cargo build -p gitjobs-server

# Format and lint server code.
server-fmt-and-lint:
    cargo fmt
    cargo check
    cargo clippy --all-targets --all-features -- --deny warnings

# Run server tests.
server-tests:
    cargo test

# Run the server with cargo watch for auto-reload.
server-watch:
    cargo watch -x "run -p gitjobs-server -- --config-file {{ config_dir }}/server.yml"

# Syncer

# Run the syncer using cargo run (builds if needed).
syncer:
    cargo run -p gitjobs-syncer -- --config-file "{{ config_dir }}/syncer.yml"

# Build the syncer binary.
syncer-build:
    cargo build -p gitjobs-syncer

# Run the syncer with cargo watch for auto-reload.
syncer-watch:
    cargo watch -x "run -p gitjobs-syncer -- --config-file {{ config_dir }}/syncer.yml"

# Frontend

e2e-tests:
    npx playwright test

# Format and lint frontend code.
frontend-fmt-and-lint:
    prettier --config gitjobs-server/static/js/.prettierrc.yaml --write "gitjobs-server/static/js/**/*.js"
    djlint --check --configuration gitjobs-server/templates/.djlintrc gitjobs-server/templates
