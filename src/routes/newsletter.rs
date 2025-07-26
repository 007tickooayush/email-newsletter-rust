use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct BodyData {
    subject: String,
    text: String,
    category: String,
}

// #[derive(serde::Deserialize)]
// pub struct Content {
//     text: String
// }

// Dummy implementation for newsletter endpoint
pub async fn publish_newsletter(_body: web::Json<BodyData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}