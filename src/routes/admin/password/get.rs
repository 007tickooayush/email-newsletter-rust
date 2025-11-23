use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use crate::session_state::TypedSession;
use crate::utils::{e500, see_other};
use std::fmt::Write;

pub async fn change_password_form(
    session: TypedSession,
    flash_messages: IncomingFlashMessages
) -> Result<HttpResponse, actix_web::Error> {

    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }

    let mut msg_html = String::new();

    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    Ok(
        HttpResponse::Ok().content_type(ContentType::html())
            .body(format!(r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                <meta http-equiv="content-type" content="text/html; charset=utf-8">
                <title>Change Password</title>
                <style>
                    body {{
                        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                        background: linear-gradient(135deg, #ece9e6, #ffffff);
                        color: #333;
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        justify-content: center;
                        height: 100vh;
                        margin: 0;
                    }}
                    h1 {{
                        color: #444;
                        margin-bottom: 20px;
                    }}
                    form {{
                        background-color: white;
                        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
                        border-radius: 8px;
                        padding: 30px 40px;
                        width: 320px;
                    }}
                    label {{
                        display: block;
                        margin-top: 15px;
                        font-weight: 500;
                        color: #555;
                    }}
                    input[type="password"] {{
                        width: 100%;
                        padding: 10px;
                        margin-top: 5px;
                        border: 1px solid #ccc;
                        border-radius: 4px;
                        font-size: 14px;
                        transition: border-color 0.3s, box-shadow 0.3s;
                    }}
                    input[type="password"]:focus {{
                        border-color: #4a90e2;
                        box-shadow: 0 0 5px rgba(74, 144, 226, 0.3);
                        outline: none;
                    }}
                    button {{
                        width: 100%;
                        padding: 12px;
                        margin-top: 25px;
                        background-color: #4a90e2;
                        color: white;
                        border: none;
                        border-radius: 4px;
                        font-size: 16px;
                        cursor: pointer;
                        transition: background-color 0.3s, transform 0.2s;
                    }}
                    button:hover {{
                        background-color: #357abd;
                        transform: translateY(-1px);
                    }}
                    button:active {{
                        transform: translateY(1px);
                    }}
                    .flash-message {{
                        background-color: #ffe5e5;
                        color: #a33;
                        padding: 10px;
                        border: 1px solid #faa;
                        border-radius: 4px;
                        margin-bottom: 15px;
                        text-align: center;
                    }}
                    .back-link {{
                        margin-top: 20px;
                        display: inline-block;
                        text-decoration: none;
                        color: #4a90e2;
                        transition: color 0.3s;
                    }}
                    .back-link:hover {{
                        color: #2c5ea8;
                    }}
                </style>
                </head>
                <body>
                <h1>Change Your Password</h1>
                {msg_html}
                <form action="/admin/password" method="post">
                    <label>Current password
                        <input
                            type="password"
                            placeholder="Enter current password"
                            name="current_password"
                        >
                    </label>
                    <label>New password
                        <input
                            type="password"
                            placeholder="Enter new password"
                            name="new_password"
                        >
                    </label>
                    <label>Confirm new password
                        <input
                            type="password"
                            placeholder="Type the new password again"
                            name="new_password_check"
                        >
                    </label>
                    <button type="submit">Change Password</button>
                </form>
                <a href="/admin/dashboard" class="back-link">&lt;- Back</a>
                </body>
                </html>
            "#
            ))
    )
}