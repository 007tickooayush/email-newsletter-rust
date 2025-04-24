
## Command to install sqls-cli using cargo
```bash
cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres
```
## Command to create a new script under `migrations` folder:
```bash
sqlx migrate add create_subscriptions_table
```