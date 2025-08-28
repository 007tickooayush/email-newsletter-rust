use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use crate::telemetry::spawn_blocking_with_tracing;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid Credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error)
}

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>
}

#[tracing::instrument(
    name = "Validate Credentials",
    skip(credentials, pool)
)]
pub async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool
) -> Result<uuid::Uuid, AuthError> {

    // Standardizing the response time for existing username and non-existing credentials
    let mut user_id = None;
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno".to_string()
    );

    if let Some((stored_user_id, stored_password_hash)) = get_stored_credentials(
        &credentials.username,
        &pool
    )
        .await?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    // This is a CPU intensive task
    // Offloaded to separate thread via custom spawn_blocking implementation
    // provided in current project's telemetry handling
    spawn_blocking_with_tracing(|| {
        // the separate thread is required, but it is also required to be in current tracing span's
        // scope, which is provided via the spawn_blocking_with_tracing_function, in order to
        // inherit the root span's(current thread's) properties, e.g, request_id, http.method,
        // http.route, etc.
        verify_password_hash(
            expected_password_hash,
            credentials.password
        )
    })
        .await
        .context("Invalid password")??;

    // The return value is only set to `Some` if the credentials are found in the store
    // Hence, even if the default password ends up matching with the provided password
    // ew never authenticate the non-existing user.
    //
    // This is also being tested by adding a test case specific to this scenario
    user_id
        .ok_or_else(|| anyhow::anyhow!("unknown username"))
        .map_err(AuthError::InvalidCredentials)
}

#[tracing::instrument(
    name = "Verify Password Hash",
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(
        &expected_password_hash.expose_secret()
    )
        .context("Failed to parse hash in PHC string format")?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash
        )
        .context("Invalid Password")
        .map_err(AuthError::InvalidCredentials)
}

#[tracing::instrument(
    name = "Get stored credentials",
    skip(username, pool)
)]
async fn get_stored_credentials(
    username: &str,
    pool: &PgPool
) -> Result<Option<(uuid::Uuid, Secret<String>)>, anyhow::Error>{
    let row = sqlx::query!(
        r#"
        SELECT user_id, password_hash
        FROM users
        WHERE username = $1
        "#,
        username
    )
        .fetch_optional(pool)
        .await
        .context("Failed to perform a query to retrieve stored credentials")?
        .map(|row| (row.user_id, Secret::new(row.password_hash)));
    Ok(row)
}