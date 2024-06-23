use sqlx::{Connection, PgConnection};
use std::net::TcpListener;
use zero2prod::configuration::get_config;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn greet_with_no_name_works() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/greet", &address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(
        response.text().await.expect("Failed to read text"),
        "Hello World!"
    );
}

#[tokio::test]
async fn greet_with_name_works() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/greet/Liam", &address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(
        response.text().await.expect("Failed to read text"),
        "Hello Liam!"
    );
}

#[tokio::test]
async fn subscribe_returns_200_with_valid_data() {
    // Arrange
    let address = spawn_app();
    let configuration = get_config().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to postgres");

    let client = reqwest::Client::new();

    // Act
    let body = "name=Liam%20Farese&email=abc%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "abc@gmail.com");
    assert_eq!(saved.name, "Liam Farese");
}

#[tokio::test]
async fn subscribe_returns_400_with_invalid_data() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=Liam%20Farese", "missing the email"),
        ("email=adc%40gmail.com", "missing name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        // Assert
        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
}

fn spawn_app() -> String {
    let listener: TcpListener =
        TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::startup::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
