use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};
use crate::helpers::{spawn_app, ConfirmationLink, TestApp};

#[tokio::test]
async fn test_newsletters_returns_400_for_invalid_data() {
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!({
                "text": "Newsletter body as plain text",
                "category": "subscribers"
            }),
            "missing subject"
        ),
        (
            serde_json::json!({
                "subject": "Newsletter subject",
                "category": "subscribers"
            }),
            "missing content"
        ),
        (
            serde_json::json!({
                "subject": "Newsletter subject",
                "text": "<p>Newsletter text content</p>"
            }),
            "missing category"
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletters(invalid_body).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
}

#[tokio::test]
async fn test_newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        // we assert that no request is fired at MailTrap
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "subject": "Newsletter title",
        "text": "<p>Newsletter body</p>",
        "category": "subscribers",
    });

    // Will return 404 if the endpoint is not added
    let response = app.post_newsletters(newsletter_request_body).await;

    // `Mock` verifies on `Drop` whether the newsletter mail is sent
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn test_newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/api/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "subject": "Newsletter title",
        "text": "<p>Newsletter body content</p>",
        "category": "subscribers"
    });
    let response = app.post_newsletters(newsletter_request_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

/// Use the Public API of the application to create an unconfirmed subscriber
async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLink {
    let body = "name=honda%20davidson&email=honda_davidson%40gmail.com";

    // Test the subscription endpoint by sending a POST request
    // This is required since the subscribe endpoint is updated to send a confirmation email
    Mock::given(path("/api/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    // inspect the requests received by the mock server MailTrap server
    // to retrieve the confirmation link and return it
    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_link(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    // Reuse the same helper function and add and extra step to
    // actually call the confirmation link
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

}