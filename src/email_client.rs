use reqwest::Client;

use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;
use crate::email_request::{FromEmailRequest, SendEmailRequest, ToEmailRequest};

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    sender_name: SubscriberName
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail, sender_name: SubscriberName) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
            sender_name
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        // html_content: &str,  // No html content required in MailTrap email schema
        text_content: &str,
        category: &str
    ) -> Result<(), String> {

        // Converting the base_url type from String to reqwest::Url
        // Enables us to use reqwest::url::join for better URL handling
        let url = format!("{}/api/send", self.base_url);

        let from = FromEmailRequest::new(
            SubscriberEmail::parse(self.sender.as_ref().to_owned())?, // using `as_ref` function as it returns directly &str from the type
            SubscriberName::parse(self.sender_name.as_ref().to_owned())?
        );

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

        let builder = self.http_client.post(&url);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence, Word};
    use fake::faker::name::en::FirstName;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::any;
    use crate::domain::subscriber_email::SubscriberEmail;
    use crate::domain::subscriber_name::SubscriberName;
    use crate::email_client::EmailClient;

    #[tokio::test]
    async fn test_send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();

        // just the first name should suffice
        let sender_name = SubscriberName::parse(FirstName().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender, sender_name);

        Mock::given(any())
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