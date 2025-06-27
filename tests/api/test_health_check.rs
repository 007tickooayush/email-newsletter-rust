use crate::helpers::spawn_app;

#[tokio::test]
async fn test_health_check() {

    //No .await and no .expect required here
    let address = spawn_app().await.address;

    let client = reqwest::Client::new();
    let response = client.get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
#[test]
fn dummy_fail() {
    // the "claim" dependency can be utilized to get details regarding
    // In case if Err is returned while testing return type `Result<Ok,Err>` and when Err is returned
    // there is no justification regarding the error, leaving all the details hidden for that test case
    let result: Result<&str, &str> = Err("This is a dummy test");
    claim::assert_err!(result);
}