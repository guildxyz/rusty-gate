#![allow(legacy_derive_helpers)]

use actix_web::{middleware::Logger, App, HttpServer};
use anyhow::Error;
use env_logger::{Builder, Env};
use log::{error, info};
use structopt::StructOpt;

mod api;
mod requirements;
mod types;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Guild API params",
    about = "Advanced parameters for the Guild Gate API."
)]
struct Opt {
    /// Set logging level
    #[structopt(short, long, default_value = "warn")]
    log: String,

    /// Set IP address
    #[structopt(long, short, default_value = "127.0.0.1")]
    ip: String,

    /// Set port number
    #[structopt(long, short, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() -> ! {
    let opt = Opt::from_args();

    Builder::from_env(Env::default().default_filter_or(opt.log)).init();

    loop {
        if let Err(e) = try_main(&opt.ip, opt.port).await {
            error!("{e}");
        } else {
            info!("Exiting gracefully");
            std::process::exit(0);
        }
    }
}

async fn try_main(ip: &str, port: u16) -> Result<(), Error> {
    info!("Listening on http://{}:{}", ip, port);

    use api::router::*;

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(check_roles_of_members)
    })
    .bind((ip, port))
    .map_err(Error::msg)?
    .run()
    .await
    .map_err(Error::msg)
}
