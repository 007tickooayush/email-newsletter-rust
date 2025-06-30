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

## Combined command to build and run the docker image:
```bash
docker build --tag email-newsletter-rust --file Dockerfile . && docker run -p 9001:9001 email-newsletter-rust
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

## MailTrap API Demo Request:
```bash
curl --location --request POST 'https://send.api.mailtrap.io/api/send' --header 'Authorization: Bearer 1c28f6aff837d05f910a7d70402fe41d' --header 'Content-Type: application/json' --data-raw '{"from":{"email":"test@ayush-tickoo.in","name":"MailTrap Test"},"to":[{"email":"test@gmail.com"}],"subject":"Email from CEO","text":"Test Email Body","category":"MailTrap Test Category"}'
```
## MailTrap API Demo Response:
```json
{"success":true,"message_ids":["8b627ff0-52b2-11f0-0000-f1e8ba0efc25"]}
```


## Migrations Script for subscriptions table:
```bash
sqlx migrate add add_status_to_subscriptions
```

## Created Script for Non-nullable status column:
```bash
sqlx migrate add make_status_not_null_in_subscriptions
```

## Script for creating `scubscription_tokens` table:
```bash
sqlx migrate add create_subscription_tokens_table
```