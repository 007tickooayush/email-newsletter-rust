use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};
use crate::helpers::{spawn_app, ConfirmationLink, TestApp};

#[tokio::test]
async fn test_newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

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
    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .json(&newsletter_request_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // `Mock` verifies on `Drop` whether the newsletter mail is sent
    assert_eq!(response.status().as_u16(), 200);
}

/// Use the Public API of the application to create an unconfirmed subscriber
async fn create_unconfirmed_subscriber_confirmation_link(app: &TestApp) -> ConfirmationLink {
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

async fn create_unconfirmed_subscriber(app: &TestApp) {
    // Reuse the same helper function and add and extra step to
    // actually call the confirmation link
    let confirmation_link = create_unconfirmed_subscriber_confirmation_link(app).await;
    reqwest::get(confirmation_link.link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

}