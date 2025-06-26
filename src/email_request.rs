use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;

#[derive(serde::Serialize)]
pub struct FromEmailRequest {
    email: SubscriberEmail,
    name: SubscriberName
}


impl FromEmailRequest {
    pub fn new(email: SubscriberEmail, name: SubscriberName) -> Self {
        Self { email, name }
    }
}

#[derive(serde::Serialize)]
pub struct ToEmailRequest {
    email: SubscriberEmail,
}

impl ToEmailRequest {
    pub fn new(email: SubscriberEmail) -> Self {
        Self { email }
    }
}

#[derive(serde::Serialize)]
// utilize if pascal case of the fields is required
// may be required in case of postmark
// #[serde(rename_all = "PascalCase")] 
pub struct SendEmailRequest {
    pub from: FromEmailRequest,
    pub to: Vec<ToEmailRequest>,
    pub subject: String,
    pub text: String,
    pub category: String
}