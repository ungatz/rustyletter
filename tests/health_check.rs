//! src/tests has all the integration tests where we try to imitate how a user might interact with out backend.
use std::net::TcpListener;
use sqlx::{PgPool, Executor, PgConnection, Connection};
use rustyletter::configuration::{get_configuration, DatabaseSettings};
use rustyletter::startup::run;
use sqlx::types::Uuid;
use sqlx::types::chrono::Utc;

pub struct TestApp{
    pub address: String,
    pub db_pool: PgPool
}

// We use TcpListner as we cannot bind to a fixed port as it may be busy
// adding :0 at the end of localhost tells the os to assign a random unused port.
async fn spawn_app() -> TestApp {
    let listner = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");
    let port = listner.local_addr().unwrap().port(); // Extracting the random assigned port
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_db(&configuration.database).await;
    
    let server = run(listner, connection_pool.clone())
        .expect("Unable to bind address!");

    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool
    }
}

// We will create new db for each test run for test isolation
// as we can't write the same values to db again and again.
pub async  fn configure_db(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create new database.");
    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database.");   
    // return PgPool
    connection_pool
}   

#[tokio::test]
async fn subscriber_returns_200_for_valid_form_data(){
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    //Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com"; // This is x-www-form-url-encoded
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute POST request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscriber_returns_400_when_data_is_missing(){
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    //Act
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];
    for(invalid_body, error_message) in test_cases {
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
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
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Request execution failed");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
