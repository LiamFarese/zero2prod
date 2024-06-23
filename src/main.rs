use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_config;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let settings = get_config().expect("Could not read configuration.yaml");
    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener: TcpListener = TcpListener::bind(address)?;
    let pool = PgPool::connect(&settings.database.connection_string())
        .await
        .expect("Failed to connect to database");
    run(listener, pool)?.await
}
