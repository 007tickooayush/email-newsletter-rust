use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use crate::helpers::spawn_app;

#[tokio::test]
async fn test_confirmation_without_token_are_rejected_with_a_400() {
    let app = spawn_app().await;

    let response = reqwest::get(
        &format!("{}/subscriptions/confirm", app.address)
    )
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn test_the_link_returned_by_subscribe_returns_a_200_if_called() {
    let app = spawn_app().await;
    let body = "name=honda%20davidson&email=honda_davidson%40gmail.com";

    Mock::given(path("/api/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let mut confirmation_link = app.get_confirmation_link(&email_request).link;

    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    // rewrite the confirmation url with port
    confirmation_link.set_port(Some(app.port)).unwrap();

    let response = reqwest::get(confirmation_link)
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn test_clicking_on_the_confirmation_link_confirms_a_subscriber() {
    let app = spawn_app().await;
    let body = "name=honda%20davidson&email=honda_davidson%40gmail.com";

    Mock::given(path("/api/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()[0];

    let confirmation_link = app.get_confirmation_link(&email_request);

    reqwest::get(confirmation_link.link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!(r#"
        SELECT email, name, status FROM subscriptions
    "#)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription along with status and token generation");

    assert_eq!(saved.email, "honda_davidson@gmail.com");
    assert_eq!(saved.name, "honda davidson");
    assert_eq!(saved.status, "confirmed");

}