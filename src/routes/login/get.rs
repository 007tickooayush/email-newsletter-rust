use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::{HttpRequest, HttpResponse};
use actix_web::http::header::ContentType;
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use std::fmt::Write;

pub async fn login_form(
    flash_messages: IncomingFlashMessages
) -> HttpResponse {
    let mut error_html = String::new();

    // iterate over all type of Flash Messages instead of only errors
    for m in flash_messages.iter() { // .filter(|m| m.level() == Level::Error)
        writeln!(error_html, "<p class='flash-msg'><i>{}</i></p>", m.content()).unwrap();
    }

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
            <!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Login</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #74ABE2, #5563DE);
            margin: 0;
            padding: 0;
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: center;
            height: 100vh;
            color: #333;
        }}

        h1 {{
            text-align: center;
            color: white;
            margin-bottom: 20px;
            text-shadow: 0 2px 5px rgba(0,0,0,0.3);
        }}

        .container {{
            background: white;
            border-radius: 10px;
            box-shadow: 0 4px 20px rgba(0,0,0,0.1);
            padding: 30px 40px;
            width: 350px;
            animation: fadeIn 1s ease;
        }}

        .flash-msg {{
            background: #ffeaea;
            color: #b30000;
            border-left: 4px solid #b30000;
            padding: 10px;
            border-radius: 5px;
            margin-bottom: 15px;
            font-size: 0.9rem;
            animation: slideDown 0.5s ease;
        }}

        form {{
            display: flex;
            flex-direction: column;
            gap: 15px;
        }}

        label {{
            display: flex;
            flex-direction: column;
            font-weight: bold;
            color: #444;
        }}

        input[type="text"],
        input[type="password"] {{
            padding: 10px;
            border: 1px solid #ccc;
            border-radius: 5px;
            transition: border-color 0.3s;
        }}

        input[type="text"]:focus,
        input[type="password"]:focus {{
            border-color: #5563DE;
            outline: none;
        }}

        button {{
            background: #5563DE;
            color: white;
            font-weight: bold;
            padding: 10px;
            border: none;
            border-radius: 5px;
            cursor: pointer;
            transition: background 0.3s;
        }}

        button:hover {{
            background: #4450bf;
        }}

        @keyframes fadeIn {{
            from {{ opacity: 0; transform: scale(0.95); }}
            to {{ opacity: 1; transform: scale(1); }}
        }}

        @keyframes slideDown {{
            from {{ opacity: 0; transform: translateY(-10px); }}
            to {{ opacity: 1; transform: translateY(0); }}
        }}

        footer {{
            margin-top: 20px;
            font-size: 0.8rem;
            color: rgba(255,255,255,0.8);
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Welcome Back</h1>
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
    </div>
    <footer>
        &copy; 2025 Customized Login Portal
    </footer>
</body>
</html>
            "#
        ))
}
