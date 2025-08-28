use std::fmt::Formatter;
use actix_web::http::header::LOCATION;
use actix_web::{web, HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use secrecy::Secret;
use sqlx::PgPool;
use crate::authentication::{validate_credentials, Credentials};
use crate::routes::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>
}

pub async fn login(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password
    };

    tracing::Span::current()
        .record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current()
                .record("user_id", &tracing::field::display(&user_id));

            // redirecting to homepage
            // For comprehensive documentation see MDN Web Docs regarding "303 See Other"
            HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish()
        }
        Err(_) => {
            // TODO: implement the error handling when the user credentials validation fails
            unimplemented!()
        }
    }

    // HttpResponse::Ok()
    //     .content_type(actix_web::http::header::ContentType::html())
    //     .body(include_str!("login.html"))
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

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match self {
            LoginError::AuthError(_) => StatusCode::UNAUTHORIZED,
            LoginError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
