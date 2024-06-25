use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_config;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("Zero2Prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let settings = get_config().expect("Could not read configuration.toml");
    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener: TcpListener = TcpListener::bind(address)?;
    let pool = PgPool::connect(settings.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to database");
    run(listener, pool)?.await
}
