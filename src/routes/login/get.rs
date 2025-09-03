use actix_web::{web, HttpResponse};
use actix_web::http::header::ContentType;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: Option<String>,
}

pub async fn login_form(query: web::Query<QueryParams>) -> HttpResponse{
    let error_html = match query.0.error {
        None => "".to_string(),
        Some(error_message) => format!("<p><i>{error_message}</i></p>")
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
            <!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Login</title>
</head>
<body>
    {error_html}
    <form method="post">
        <label>
            Username
            <input
                    type="text"
                    placeholder="Enter username"
                    name="username"
            >
        </label>
        <label>
            Password
            <input
                    type="password"
                    placeholder="Enter password"
                    name="password"
            >
        </label>

        <button type="submit">Login</button>
    </form>
</body>
</html>
            "#
        ))
}