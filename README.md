# Email Newsletter Rust

## NOTE: Run the [init_db.sh](./scripts/init_db.sh) before building/running the project else there will be sqlx expand error

## Command to install sqls-cli using cargo
```bash
cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres
```
## Command to create a new script under `migrations` folder:
```bash
sqlx migrate add create_subscriptions_table
```

## When you want to see all logs coming out of a certain test case to debug it you can run
```bash
# We are using the `bunyan` CLI to prettify the outputted logs
# The original `bunyan` requires NPM, but you can install a Rust-port with
# `cargo install bunyan`
TEST_LOG=true cargo test health_check_works | bunyan
```

## It must be invoked as a cargo subcommand All options after `--` are passed to cargo itself 
## We need to point it at our library since it contains all our SQL queries.
```bash
cargo sqlx prepare -- --lib
```

## Build a docker image tagged as "email-newsletter-rust" according to the recipe specified in `Dockerfile`
```bash
docker build --tag email-newsletter-rust --file Dockerfile .
```

## Run the docker image
```bash
docker run -p 9001:9001 email-newsletter-rust
```

# Remove unused Docker objects (containers, images, volumes, etc.):
```bash
docker system prune -a --volumes
```

## Get the size of our docker image:
```bash
docker image inspect email-newsletter-rust --format='{{.Size}}'
```

# Testing Commands

## Health check:
```bash
curl http://localhost:9001/health_check
```

## Subscribe to a newsletter:
```bash
curl --request POST \
--data 'name=le%20guin&email=ursula_le_guin%40gmail.com' \
http://localhost:9001/subscriptions --verbose
``` 