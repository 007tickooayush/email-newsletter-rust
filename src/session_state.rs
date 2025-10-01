use std::future::{ready, Ready};
use actix_session::{Session, SessionExt, SessionGetError, SessionInsertError};
use actix_web::{FromRequest, HttpRequest};
use actix_web::dev::Payload;
use uuid::Uuid;

pub struct TypedSession(Session);

impl TypedSession {
    const USER_ID_KEY: &'static str = "user_id";

    pub fn renew(&self) {
        self.0.renew();
    }

    pub fn insert_user_id(&self, user_id: Uuid) -> Result<(), SessionInsertError> { // serde_json::Error
        self.0.insert(Self::USER_ID_KEY, user_id)
    }

    pub fn get_user_id(&self) -> Result<Option<Uuid>, SessionGetError> { // serde_json::Error
        self.0.get(Self::USER_ID_KEY)
    }
}

impl FromRequest for TypedSession {

    // Returning the same error as FromRequest for Session
    type Error = <Session as FromRequest>::Error;

    // async trait not yet supported in Rust
    // Wrap the `TypedSession` in a `Ready` future  that resolves the wrapped `Session`
    // the first time it is polled by the executor
    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}