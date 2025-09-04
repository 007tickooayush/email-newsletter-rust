use std::fmt::Formatter;
use actix_web::http::header::LOCATION;
use actix_web::{web, HttpResponse, ResponseError};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use hmac::{Hmac, Mac};
use secrecy::Secret;
use sqlx::PgPool;
use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::routes::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>
}

pub async fn login(form: web::Form<FormData>, pool: web::Data<PgPool>) -> Result<HttpResponse, LoginError> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password
    };

    tracing::Span::current()
        .record("username", &tracing::field::display(&credentials.username));

    let user_id = validate_credentials(credentials, &pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into())
        })?;

    // HttpResponse::Ok()
    //     .content_type(actix_web::http::header::ContentType::html())
    //     .body(include_str!("login.html"))
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish())
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication Failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error)
}


impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

/// Implemented Simple Redirect for handling the request in case of any error
impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        StatusCode::SEE_OTHER
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let query_string = format!(
            "error={}",
            urlencoding::Encoded::new(self.to_string())
        );


        // TODO: handle the private key required for hmac's sha2
        let secret: &[u8] = &Vec::new();

        let hmac_tag = {
            // Handling the Message Authentication Code (MAC)
            let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret).unwrap();
            mac.update(query_string.as_bytes());
            mac.finalize().into_bytes()
        };

        let encoded_error = urlencoding::Encoded::new(self.to_string());

        // Appended hexadecimal representation of HMAC tag to the query string
        // as an additional query parameter
        HttpResponse::build(self.status_code())
            .insert_header((
                LOCATION,
                format!("/login?error={query_string}&tag={hmac_tag:x}", encoded_error)
            ))
            .finish()
    }
}
