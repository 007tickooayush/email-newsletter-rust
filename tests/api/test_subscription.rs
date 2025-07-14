use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use crate::helpers::spawn_app;

#[tokio::test]
async fn test_subscribe_returns_200_for_valid_form_data() {

    // get the Application struct which includes the Connection Pool object, directly
    let app = spawn_app().await;
    let body = "name=honda%20davidson&email=honda_davidson%40gmail.com";

    // Test the subscription endpoint by sending a POST request
    // This is required since the subscribe endpoint is updated to send a confirmation email
    Mock::given(path("/api/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {

    // get the Application struct which includes the Connection Pool object, directly
    let app = spawn_app().await;
    let body = "name=honda%20davidson&email=honda_davidson%40gmail.com";
    Mock::given(path("/api/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)// use the db_pool object directly from the app
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "honda_davidson@gmail.com");
    assert_eq!(saved.name, "honda davidson");
    assert_eq!(saved.status, "pending_confirmation");;
}

#[tokio::test]
async fn test_subscribe_returns_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=le%20guin%40gmail.com", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing name"),
        ("", "missing email and name both"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "API did not fail with 400 error code: {}",
            error_message
        );
    }
}

#[tokio::test]
async fn test_subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;

    let body = "name=honda%20davidson&email=honda_davidson%40gmail.com";

    Mock::given(path("/api/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        // Not settings any expectations here
        // .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app
        .email_server
        // Method of MockServer to intercept requests and receive a vector of `Request` object
        .received_requests()
        .await
        .unwrap()[0];
    let confirmation_link = app.get_confirmation_link(&email_request).link;

    assert!(!confirmation_link.host_str().unwrap().is_empty());
}

/// # sqlx logs are a bit spammy, cutting them out to reduce noise
/// export RUST_LOG="sqlx=error,info"<br/>
/// export TEST_LOG=enabled<br/>
/// run `cargo install bunyan` before the test for getting more accuracy
/// cargo test --package email-newsletter-rust --test api test_subscription::test_subscribe_fails_if_there_is_a_fatal_database_error -- --exact | bunyan
#[tokio::test]
async fn test_subscribe_fails_if_there_is_a_fatal_database_error() {
    let app = spawn_app().await;
    let body = "name=honda%20davidson&email=honda_davidson%40gmail.com";

    // Sabotage the database
    sqlx::query!(
        "ALTER TABLE subscription_tokens DROP COLUMN subscription_token"
    )
        .execute(&app.db_pool)
        .await
        .unwrap();

    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(response.status().as_u16(), 500);
}
