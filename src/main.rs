use env_logger::Env;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_config;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let settings = get_config().expect("Could not read configuration.toml");
    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener: TcpListener = TcpListener::bind(address)?;
    let pool = PgPool::connect(&settings.database.connection_string())
        .await
        .expect("Failed to connect to database");
    run(listener, pool)?.await
}
