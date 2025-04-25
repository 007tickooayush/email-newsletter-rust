use actix_web::{web, HttpResponse};
use sqlx::PgConnection;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String
}

pub async fn subscribe(
    _form: web::Form<FormData>,
    // Retrieving a connection from the application state!
    _connection: web::Data<PgConnection>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}