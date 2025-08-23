use actix_web::HttpResponse;

pub async fn login() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(actix_web::http::header::ContentType::html())
        .body(include_str!("login.html"))
}