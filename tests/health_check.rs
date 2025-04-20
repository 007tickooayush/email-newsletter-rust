use std::net::TcpListener;

#[tokio::test]
async fn test_health_check() {
 
    //No .await and no .expect required here
    let address = spawn_app();
 
    let client = reqwest::Client::new();
    let response = client.get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request");
    
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    // We are not using the port 9001 here, instead we are binding to a random port provided by OS
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    
    // Here we dont .await the call, instead run the process in the background using tokio::spawn function
    // and return the server handle
    let server = email_newsletter_rust::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    
    format!("http://127.0.0.1:{}", port)
}