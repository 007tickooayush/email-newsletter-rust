use sqlx::{PgConnection, Connection, Executor};
use crate::helpers::spawn_app;

#[tokio::test]
async fn test_subscribe_returns_200_for_valid_data() {
    let test_config = spawn_app().await;

    // let configuration = get_configuration().expect("Failed to get Configuration!");
    let configuration = test_config.configuration;
    let db_conn = configuration.database.with_db();

    // The "Connection" trait must be in scope to invoke
    // `PgConnection::connect`
    // it is not an inherent method of the struct
    // hence we also need to import `Connection` trait from sqlx
    let mut connection = PgConnection::connect_with(&db_conn)
        .await
        .expect("Failed to connect to Postgres");

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &test_config.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn test_subscribe_returns_400_when_fields_are_present_but_invalid() {
    let address = spawn_app().await.address;
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

