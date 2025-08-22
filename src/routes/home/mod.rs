use actix_web::HttpResponse;

pub async fn home() -> HttpResponse {
    // include_str!() function operates as compile time, i.e, the read file content is also stored
    // as a part of application's binary, and the pointer to its content remains valid indefinitely
    // as 'static string slice (&'static str) in form of UTF-8 encoded string
    HttpResponse::Ok()
        .content_type(actix_web::http::header::ContentType::html())
        .body(include_str!("home.html"))
}