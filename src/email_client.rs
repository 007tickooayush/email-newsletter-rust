use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;
use crate::email_request::{FromEmailRequest, SendEmailRequest, ToEmailRequest};

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    sender_name: SubscriberName,
    authorization_token: Secret<String>
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        sender_name: SubscriberName,
        authorization_token: Secret<String>,
        timeout: std::time::Duration
    ) -> Self {
        // Complete Client-wide timeout configuration
        // As opposed to request only timeout
        let http_client = Client::builder()
            .timeout(timeout) // Set a timeout for the HTTP client
            .build()
            .unwrap();
        Self {
            http_client,
            base_url,
            sender,
            sender_name,
            authorization_token
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        // html_content: &str,  // No html content required in MailTrap email schema
        text: &str,
        category: &str
    ) -> Result<(), reqwest::Error> {

        // Converting the base_url type from String to reqwest::Url
        // Enables us to use reqwest::url::join for better URL handling
        let url = format!("{}/api/send", self.base_url);

        // using `as_ref` function as it returns directly &str from the type
        let from_email = SubscriberEmail::parse(self.sender.as_ref().to_owned()).expect("Send Attempt for anInvalid Email");
        let from_name = SubscriberName::parse(self.sender_name.as_ref().to_owned()).expect("Send Attempt for an Invalid Name");
        let from = FromEmailRequest::new(from_email, from_name);

        let to = vec![
            ToEmailRequest::new(recipient),
        ];

        let request_body = SendEmailRequest {
            from,
            to,
            subject,
            text,
            category
        };

        let builder = self
            .http_client
            .post(&url)
            .header("Authorization", self.authorization_token.expose_secret())
            .json(&request_body)

            // Uncomment the line below to timeout the request
            // .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};
    use fake::{Fake, Faker};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence, Word};
    use fake::faker::name::en::FirstName;
    use secrecy::Secret;
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};
    use wiremock::matchers::{any, header, header_exists, method, path};
    use crate::domain::subscriber_email::SubscriberEmail;
    use crate::domain::subscriber_name::SubscriberName;
    use crate::email_client::EmailClient;

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            // NOTE: PascalCase is only required in case of PostMark not in case of MailTrap
            if let Ok(body) = result {
                // Uncomment the line below to debug the request body
                // dbg!(&body);

                // Implementing the matcher's boolean result based on MailTrap's email schema
                // All the mandatory fields are checked
                body.get("from").is_some() &&
                    body.get("to").is_some() &&
                    body.get("subject").is_some() &&
                    body.get("text").is_some() &&
                    body.get("category").is_some()
            } else {
                false
            }
        }
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }
    /// Generate a random email content
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    /// Generate a random subscriber email
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn category() -> String {
        Word().fake()
    }

    fn sender_name() -> SubscriberName {
        SubscriberName::parse(FirstName().fake()).unwrap()
    }

    /// Get a test instance of `EmailClient`.
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            sender_name(),
            Secret::new(Faker.fake()),
            // lesser amount for testing
            std::time::Duration::from_millis(200)
        )
    }

    #[tokio::test]
    async fn test_send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("Authorization")) // in Postmark "X-Postmark-Server-Token" is utilized
            .and(header("Content-type", "application/json"))
            .and(path("/api/send")) // in Postmark path "/email" is utilized
            .and(method("POST"))
            // utilizing the Email body matcher
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;


        let outcome = email_client
            .send_email(email(), &subject(), &content(), &category())
            .await;

        // Assert that the request was sent successfully
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn test_send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();
        let category: String = Word().fake();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(subscriber_email, &subject, &content, &category)
            .await;

        assert_err!(outcome);
    }

    #[tokio::test]
    async fn test_email_times_out_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let response = ResponseTemplate::new(200)
            // set the delay of 3 minutes
            .set_delay(std::time::Duration::from_millis(300));

        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &category())
            .await;

        assert_err!(outcome);
    }
}