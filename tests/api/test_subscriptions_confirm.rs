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
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);

        links[0].as_str().to_owned()
    };

    let raw_confirmation_link = &get_link(&body["text"].as_str().unwrap());
    let confirmation_link = reqwest::Url::parse(raw_confirmation_link).unwrap();

    assert_eq!(confirmation_link.as_str(), "127.0.0.1");

    let response = reqwest::get(confirmation_link)
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
}