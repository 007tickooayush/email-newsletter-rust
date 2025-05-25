# Using the Stable Rust Build
FROM rust:1.81.0

# Creaing `app` folder in case it does not exist to utilize for our container
WORKDIR /app

# Install the required libraries and linkers for Rust
RUN apt update && apt install lld clang -y

# Copy all the contents from working env to current docker image
COPY . .

# SQLX_OFFLINE environment variable to true in our Dockerfile to force sqlx to look at the saved metadata instead of
# trying to query a live database
ENV SQLX_OFFLINE true

# Build the Rust project
# Create a release build of the Rust project
RUN cargo build --release

# When `docker run` is executed, launch the binary built by the cargo build command
ENTRYPOINT [".target/release/email-newsletter-rust"]