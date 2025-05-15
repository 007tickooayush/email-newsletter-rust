
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