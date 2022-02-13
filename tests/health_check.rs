//! src/tests has all the integration tests where we try to imitate how a user might interact with out backend.
use std::net::TcpListener;

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
