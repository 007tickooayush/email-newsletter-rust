use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry};
use tracing::Subscriber;
use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;

/// Composed multiple layers into `tracing`'s Subscriber
///
/// # USAGE:
/// We are using `impl Subscriber` as return type to avoid having to explicitly tell the
/// return type of Subscriber returned by the function.
/// We also call out the returned Value to be extending `Send` and `Sync` as it is
/// required for the `init_subscriber` function.
pub fn get_subscriber(
    name: String,
    env_filter: String
) -> impl Subscriber + Send + Sync {

    // Printing all spans at info-level
    // If the RUST_LOG env variable has not been set
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::from(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(
        name, // "email_newsletter_rust".into(),
        // Output the logs into the stdout
        std::io::stdout
    );

    // the `with` function is provided by `SubscriberExt`, an extension trait
    // for `Subscriber` exposed by `tracing_subscriber`
    Registry::default()
        // with from `layer::SubscriberExt` trait
        .with(env_filter)
        // implementing JSON based logging foe Elasitcsearch-friendly architecture
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    // `tracing::subscriber::set_global_default` is utilized to specify the subscriber for span processing
    set_global_default(subscriber).expect("Failed to set Global Subscriber");
}