use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use zero2prod::configuration::get_config;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("Zero2Prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let settings = get_config().expect("Could not read configuration.toml");
    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );

    let listener: TcpListener = TcpListener::bind(address)?;
    let pool = PgPoolOptions::new()
        .connect_lazy_with(settings.database.with_db());
    run(listener, pool)?.await
}
