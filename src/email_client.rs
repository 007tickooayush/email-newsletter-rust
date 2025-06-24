use reqwest::Client;
use crate::domain::subscriber_email::SubscriberEmail;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_send_email_fires_a_request_to_base_url() {
        todo!()
    }
}