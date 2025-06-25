use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;

pub struct FromEmailRequest {
    email: SubscriberEmail,
    name: SubscriberName
}


impl FromEmailRequest {
    pub fn new(email: SubscriberEmail, name: SubscriberName) -> Self {
        Self { email, name }
    }
}

pub struct ToEmailRequest {
    email: SubscriberEmail,
}

impl ToEmailRequest {
    pub fn new(email: SubscriberEmail) -> Self {
        Self { email }
    }
}

pub struct SendEmailRequest {
    pub from: FromEmailRequest,
    pub to: Vec<ToEmailRequest>,
    pub subject: String,
    pub text: String,
    pub category: String
}