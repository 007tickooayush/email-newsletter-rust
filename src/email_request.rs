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
pub struct SendEmailRequest {
    pub from: FromEmailRequest,
    pub to: Vec<ToEmailRequest>,
    pub subject: String,
    pub text: String,
    pub category: String
}