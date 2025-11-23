use actix_web::{web, HttpResponse};
use actix_web::http::header::{ContentType, LOCATION};
use sqlx::PgPool;
use uuid::Uuid;
use anyhow::Context;
use crate::session_state::TypedSession;
use crate::utils::e500;
// required for get_username anyhow::Error handling

pub async fn admin_dashboard(
    session: TypedSession,
    pool: web::Data<PgPool>
) -> Result<HttpResponse, actix_web::Error> {
    let username = if let Some(user_id) = session.get_user_id().map_err(e500)? {
        get_username(user_id, &pool).await.map_err(e500)?
    } else {
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/login"))
            .finish());
    };
    Ok(
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(format!(
                r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta http-equiv="content-type" content="text/html; charset=utf-8">
                    <title>Admin dashboard</title>
                    <style>
                        body {{
                            margin: 0;
                            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                            background: linear-gradient(135deg, #f5f7fa, #c3cfe2);
                            color: #333;
                        }}
                        header {{
                            background-color: #2c3e50;
                            color: white;
                            padding: 1em 2em;
                            box-shadow: 0 2px 6px rgba(0, 0, 0, 0.1);
                        }}
                        h1 {{
                            margin: 0;
                            font-size: 1.8em;
                        }}
                        main {{
                            max-width: 600px;
                            margin: 50px auto;
                            background: white;
                            border-radius: 12px;
                            box-shadow: 0 6px 12px rgba(0,0,0,0.1);
                            padding: 2em;
                            text-align: center;
                        }}
                        p {{
                            font-size: 1.1em;
                        }}
                        ol {{
                            list-style-type: none;
                            padding: 0;
                            margin-top: 1.5em;
                        }}
                        li {{
                            margin: 10px 0;
                        }}
                        a, input[type="submit"] {{
                            display: inline-block;
                            padding: 10px 20px;
                            border-radius: 8px;
                            border: none;
                            text-decoration: none;
                            font-weight: bold;
                            transition: background-color 0.3s ease, transform 0.2s ease;
                        }}
                        a {{
                            background-color: #3498db;
                            color: white;
                        }}
                        a:hover {{
                            background-color: #2980b9;
                            transform: scale(1.05);
                        }}
                        input[type="submit"] {{
                            background-color: #e74c3c;
                            color: white;
                            cursor: pointer;
                        }}
                        input[type="submit"]:hover {{
                            background-color: #c0392b;
                            transform: scale(1.05);
                        }}
                        footer {{
                            text-align: center;
                            padding: 1em;
                            font-size: 0.9em;
                            color: #555;
                        }}
                    </style>
                </head>
                <body>
                    <header>
                        <h1>Admin Dashboard</h1>
                    </header>
                    <main>
                        <p>Welcome, <strong>{username}</strong>!</p>
                        <p>Select an action below to manage your account:</p>
                        <ol>
                            <li><a href="/admin/password">Change password</a></li>
                            <li>
                                <form name="logoutForm" action="/admin/logout" method="post">
                                    <input type="submit" value="Logout" />
                                </form>
                            </li>
                        </ol>
                    </main>
                    <footer>
                        <p>&copy; 2025 Ayush Tickoo | Secure Admin Panel</p>
                    </footer>
                </body>
                </html>
                "#
            ))
    )
}

#[tracing::instrument(
    name = "Get username",
    skip(pool)
)]
pub async fn get_username(
    user_id: Uuid,
    pool: &PgPool
) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
            SELECT username
            FROM users
            WHERE user_id = $1
        "#,
        user_id
    )
        .fetch_one(pool)
        .await
        .context("failed to fetch username")?;
    Ok(row.username)
}
