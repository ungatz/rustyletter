//! src/tests has all the integration tests where we try to imitate how a user might interact with out backend.
use std::net::TcpListener;
use actix_web::{HttpMessage, ResponseError};

#[tokio::test]
async fn subscriber_returns_200_for_valid_form_data(){
    // Arrange
    let addr = spawn_app();
    let client = reqwest::Client::new();

    //Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com"; // This is x-www-form-url-encoded
    let response = client
        .post(&format!("{}/subscriptions", &addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute POST request.");

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscriber_returns_400_when_data_is_missing(){
    // Arrange
    let addr = spawn_app();
    let client = reqwest::Client::new();

    //Act
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];
    for(invalid_body, error_message) in test_cases {
    let response = client
        .post(&format!("{}/subscriptions", &addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_body)
        .send()
        .await
        .expect("Failed to execute POST request.");

        // Assert
        assert_eq!(400, response.status().as_u16(),
                   "This API did not fail with 404 error when payload was {}",
                   error_message);
    }
}

#[tokio::test]
async fn health_check_works() {
    // First we need to start the app
    let addr = spawn_app();
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", &addr))
        .send()
        .await
        .expect("Request execution failed");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
// We use TcpListner as we cannot bind to a fixed port as it may be busy
// adding :0 at the end of localhost tells the os to assign a random unused port.
fn spawn_app() -> String{
    let listner = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");
    let port = listner.local_addr().unwrap().port(); // Extracting the random assigned port
    let server = rustyletter::run(listner).expect("Unable to bind address!");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
