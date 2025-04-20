use std::net::TcpListener;
use email_newsletter_rust::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    
    let listener = TcpListener::bind("0.0.0.0:9001")?;
    
    // Bubble up the io::Error if we failed to bind the address
    // Or else just .await on Server
    run(listener)?.await
}
