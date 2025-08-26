use actix_web::http::header::LOCATION;
use actix_web::{web, HttpResponse};
use secrecy::Secret;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>
}

pub async fn login(_form: web::Form<FormData>) -> HttpResponse {
    // HttpResponse::Ok()
    //     .content_type(actix_web::http::header::ContentType::html())
    //     .body(include_str!("login.html"))

    // redirecting to homepage
    // For comprehensive documentation see MDN Web Docs regarding "303 See Other"
    HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish()
}