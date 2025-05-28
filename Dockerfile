# Using the Stable Rust Build
# -- Builder stage
FROM rust:1.81.0 AS builder
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


# -- Runtime Stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app
# Install OpenSSL - Dynamical linking is used in the Rust project
# Install ca-certificates -  Required to verify TLS certificates when establishing HTTPS connections
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copying the compiled binary from the builder environment to present environment
COPY --from=builder /app/target/release/email-newsletter-rust-bin1 email-newsletter-rust-bin1
# Copy the configuration
COPY configuration configuration
ENV APP_ENVIRONMENT production
# When `docker run` is executed, launch the binary built by the cargo build command
ENTRYPOINT ["./email-newsletter-rust-bin1"]