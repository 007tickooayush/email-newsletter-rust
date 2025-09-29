use actix_session::Session;
use actix_web::HttpResponse;
use uuid::Uuid;

pub async fn admin_dashboard(
    session: Session
) -> Result<HttpResponse, actix_web::Error> {
    let _username = if let Some(user_id) = session
        .get::<Uuid>("user_id")
        .map_err(e500)?
    {
        todo!()
    } else {
        todo!()
    };
    Ok(HttpResponse::Ok().finish())
}

/// Function to return an opaque 500 while logging the cause
fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static
{
    actix_web::error::ErrorInternalServerError(e)
}