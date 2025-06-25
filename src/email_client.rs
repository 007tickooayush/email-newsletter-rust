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
        authorization_token: Secret<String>
    ) -> Self {
        Self {
            http_client: Client::new(),
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
        text_content: &str,
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
            subject: subject.to_owned(),
            text: text_content.to_owned(),
            category: category.to_owned()
        };

        let builder = self
            .http_client
            .post(&url)
            .header("Authorization", self.authorization_token.expose_secret())
            .json(&request_body)
            .send()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence, Word};
    use fake::faker::name::en::FirstName;
    use secrecy::Secret;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{any, header, header_exists, method, path};
    use crate::domain::subscriber_email::SubscriberEmail;
    use crate::domain::subscriber_name::SubscriberName;
    use crate::email_client::EmailClient;

    #[tokio::test]
    async fn test_send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();

        // just the first name should suffice
        let sender_name = SubscriberName::parse(FirstName().fake()).unwrap();
        let email_client = EmailClient::new(
            mock_server.uri(),
            sender,
            sender_name,
            Secret::new(Faker.fake())
        );

        Mock::given(header_exists("Authorization")) // in Postmark "X-Postmark-Server-Token" is utilized
            .and(header("Content-type", "application/json"))
            .and(path("/api/send")) // in Postmark path "/email" is utilized
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();
        let category: String = Word().fake();

        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &category)
            .await;
    }
}