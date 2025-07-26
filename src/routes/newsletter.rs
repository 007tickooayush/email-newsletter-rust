use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct BodyData {
    subject: String,
    text: Content
}

#[derive(serde::Deserialize)]
pub struct Content {
    text: String
}

// Dummy implementation for newsletter endpoint
pub async fn publish_newsletter(_body: web::Data<BodyData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}