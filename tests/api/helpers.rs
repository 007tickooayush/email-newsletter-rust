use std::net::TcpListener;
use once_cell::sync::Lazy;
use uuid::Uuid;
use email_newsletter_rust::configuration::{get_configuration, DatabaseSettings, Settings};
use email_newsletter_rust::email_client::EmailClient;
use email_newsletter_rust::telemetry::{get_subscriber, init_subscriber};
use sqlx::{PgConnection, Connection, PgPool, Executor};
use wiremock::MockServer;
use email_newsletter_rust::startup::{get_connection_pool, Application};

// Ensure that the `tracing` stack is only initialized once rather than for each test case
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_lvl = "info".into();
    let subscriber_name = "test".into();

    // Cannot assign the output of `get_subscriber` to a variable based on the value of `TEST_LOG`
    // because the sink is part of the type returned by `get_subscriber`, therefore they are not the
    // same type. To work around it, but this is the most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_lvl, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_lvl, std::io::sink);
        init_subscriber(subscriber);
    }
});

/// This is implementation according to the MailTrap API not PostMark API
/// hence instead of using html and text field we only utilize a single `link` field
pub struct ConfirmationLink {
    pub link: reqwest::Url
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub configuration: Settings,
    pub email_server: MockServer,
    pub port: u16
}

impl TestApp {
    /// Create a new subscriber by sending a POST request to the `/subscriptions` endpoint
    pub async fn post_subscriptions(
        &self,
        body: String
    ) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_link(&self, email_request:&wiremock::Request) -> ConfirmationLink {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);

            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();

            // Enforce localhost call on server
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let link = get_link(&body["text"].as_str().unwrap());
        ConfirmationLink {
            link
        }
    }

    pub async fn post_newsletters(
        &self,
        body: serde_json::Value
    ) -> reqwest::Response{
        let (username, password) = self.test_user().await;
        reqwest::Client::new()
            .post(&format!("{}/newsletters", &self.address))
            // Providing Generated Credentials here
            // `reqwest` handles all the encoding/formatting
            .basic_auth(username, Some(password))
            .json(&body)
            .send()
            .await
            .expect("Failed to trigger newsletter request.")
    }

    pub async fn test_user(&self) -> (String, String) {
        let row = sqlx::query!(
            r#"SELECT username, password FROM users LIMIT 1"#
        )
            .fetch_one(&self.db_pool)
            .await
            .expect("Failed to create and fetch test users");

        (row.username, row.password)
    }
}


/// Spin up the application in the background
/// Return the address of the application i.e localhost:XXXX
pub async fn spawn_app() -> TestApp {

    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // Next invocations get skipped
    Lazy::force(&TRACING);

    // Launch mock server to stand in for MailTrap API
    let email_server = MockServer::start().await;

    // Randomized the configuration to ensure test isolation
    let configuration = {
        // Create db connection using PgPool(Pool) implementation of sqlx
        // randomize the db mame and use it for testing
        let mut c = get_configuration().expect("Failed to get Configuration in spawn_app");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;

        // Use the mock server's URI as the base URL for the email client
        c.email_client.base_url = email_server.uri();
        c
    };

    // add the removed configure_database function
    // this function will create a new database with the name
    configure_database(&configuration.database).await;

    // Here we dont .await the call, instead run the process in the background using tokio::spawn function
    // and return the server handle

    // Launch the server using the configuration built
    let application = Application::build(configuration.clone()) // utilizing .clone() to avoid moving the configuration
        .await
        .expect("Failed to build server");

    let application_port = application.port();
    let address = format!(
        "http://127.0.0.1:{}"
        ,application_port
    );
    let _ = tokio::spawn(application.run_until_stopped());

    // Get the address of the server
    let test_app = TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
        configuration,
        email_server,
        port: application_port
    };
    add_test_user(&test_app.db_pool).await;
    test_app
}

async fn add_test_user(pool: &PgPool) {
    sqlx::query!(r#"
        INSERT INTO users(user_id, username, password)
        VALUES ($1, $2, $3)
    "#,
        Uuid::new_v4(),
        Uuid::new_v4().to_string(),
        Uuid::new_v4().to_string()
    )
        .execute(pool)
        .await
        .expect("Failed to create test user");
}


/// NOTE: there is no cleanup mechanism implemented the created databases using random V4 UUIDs
/// For handling the complete process properly the active Database connections need to be terminated,
/// And the databases need to be dropped
/// ---
/// That can be achieved using the "sqlx::test" macro
///
/// For more information regarding the issue check:
/// https://stackoverflow.com/questions/73013414/drop-database-on-drop-using-sqlx-and-rust
///
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to establish connection in configure_database");

    connection
        .execute(format!(r#"
            CREATE DATABASE "{}";
        "#, config.database_name).as_str())
        .await
        .expect("FAILED to CREATE DATABASE configure_database");

    // Database migrations
    let db_conn_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to bind address for db spawn_app");

    sqlx::migrate!("./migrations")
        .run(&db_conn_pool)
        .await
        .expect("Failed to exectute migration of database");

    db_conn_pool
}