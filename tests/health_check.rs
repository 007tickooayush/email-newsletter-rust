use std::net::TcpListener;

#[tokio::test]
async fn test_health_check() {

    //No .await and no .expect required here
    let address = spawn_app();

    let client = reqwest::Client::new();
    let response = client.get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn test_subscribe_returns_200_for_valid_data() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let body = "name==le%20guin%email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_subscribe_returns_400_for_missing_data() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin%40gmail.com", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing name"),
        ("", "missing email and name both"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "API did not fail with 400 error code: {}",
            error_message
        );
    }
}

/// Spin up the application in the background
/// Return the address of the application i.e localhost:XXXX
fn spawn_app() -> String {
    // We are not using the port 9001 here, instead we are binding to a random port provided by OS
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    // Here we dont .await the call, instead run the process in the background using tokio::spawn function
    // and return the server handle
    let server = email_newsletter_rust::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}