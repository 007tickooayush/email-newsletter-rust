use std::{collections::HashSet, str::FromStr};

use reqwest::header::HeaderValue;

use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn test_an_error_flash_message_is_set_on_failure() {
    let app = spawn_app().await;

    // Part 1 - Try to login
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 303);

    // Part 2 - follow the redirect
    assert_is_redirect_to(&response, "/login");

    let html_page = app.get_login_html().await;
    println!("{}", &html_page);
    assert!(html_page.contains("<p><i>Authentication failed</i></p>"));

    // Part 3 - Reload the login page
    let html_page = app.get_login_html().await;
    assert!(!html_page.contains("<p><i>Authentication failed</i></p>"));
}

#[tokio::test]
async fn test_redirect_to_admin_dashboard_after_login_success() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    });

    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));

}