use std::net::TcpListener;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web::web::Data;
use sqlx::{PgPool};
use sqlx::postgres::PgPoolOptions;
use tracing_actix_web::TracingLogger;
use crate::configuration::{get_configuration, DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{confirm, health_check, publish_newsletter, subscribe};
use crate::telemetry::{get_subscriber, init_subscriber};

/// A new type for the application server
/// wrap actix_web::dev::Server
/// in a new type that holds on to the information we want
pub struct Application {
    port: u16,
    server: Server
}

impl Application {

    /// Converted the build function to a constructor for the `Application`
    pub async fn build(
        configuration: Settings
    ) -> Result<Self, std::io::Error> {
        // Moved te startup initialization logic to a separate function

        // Using Pool implementation in order to handle concurrency of database query executions
        // only try to establish a connection when the pool is used for the first time.
        let connection = PgPoolOptions::new()
            .connect_timeout(std::time::Duration::from_secs(2))
            .connect_lazy_with(configuration.database.with_db());

        // A new `EmailClient` created using `configuration`
        let sender_email = configuration.email_client.sender()
            .expect("Invalid Sender Email Address");
        let sender_name = configuration.email_client.sender_name()
            .expect("Invalid Sender Name");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            sender_name,
            configuration.email_client.authorization_token,
            timeout
        );

        // Remove the hardcoded 9001 port
        let address = format!("{}:{}", configuration.application.host , configuration.application.port);
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();
        // Storing the actix::Server object
        let server = run(
            listener,
            connection,
            email_client,
            configuration.application.base_url
        )?;

        // Save the port in the Application's port attribute
        Ok( Self {
            port,
            server
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// this function only returns when the application is stopped.
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

/// Helper function to get the Database Connection Pool Object
pub fn get_connection_pool(
    configuration: &DatabaseSettings
) -> PgPool {
    // Using Pool implementation in order to handle concurrency of database query executions
    // only try to establish a connection when the pool is used for the first time.
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}


// We need to define a wrapper type in order to retrieve the URL
// in the `subscribe` handler.
// Retrieval from the context, in actix-web, is type-based: using
// a raw `String` would expose us to conflicts.
pub struct ApplicationBaseUrl(pub String);


pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String
) -> Result<Server, std::io::Error> {

    // using web::Data to wrap the connection in smart pointer(Arc)
    // as App required the app_data to implement Clone trait for "T"
    // and in Arc<T> T is clonable, no matter what T is
    let connection = web::Data::new(db_pool);

    // Wrap the email client in web::Data to share it across requests
    let email_client = web::Data::new(email_client);

    let base_url = Data::new(ApplicationBaseUrl(base_url));

    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(publish_newsletter))
            // use `TracingLogger` provided by `tracing-actix-web` crate instead of `Logger` of actix_web crate
            .wrap(TracingLogger::default())
            // Added email_client to application state/context
            .app_data(email_client.clone())
            .app_data(connection.clone())
            .app_data(base_url.clone())
    })
        .listen(listener)?
        .run();
    // No .await here
    Ok(server)
}