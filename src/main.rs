mod pages;

use actix_web::{middleware, web, App, HttpServer};
use clap::Parser;
use env_logger::Env;
use log::info;
use web_hook::AppData;

const NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    // Work Dir
    #[clap(short, long, default_value_t = format!("./logs/{}", NAME))]
    dir: String,
    // Secret Key
    #[clap(short, long, default_value_t = String::from("12345"))]
    secret: String,
    // Allowed User Agent
    #[clap(short, long, default_value_t = String::from("foobar"))]
    ua: String,
    // Service Port
    #[clap(short, long, default_value_t = 8080)]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let env = Env::default().filter_or("LOG_LEVEL", "debug");
    env_logger::init_from_env(env);

    HttpServer::new(|| {
        let cli = Cli::parse();
        let data = web::Data::new(AppData {
            dir: cli.dir,
            secret: cli.secret,
            ua: cli.ua,
        });
        info!("Starting HTTP server at http://localhost:{}", cli.port);
        App::new()
            // store in application storage
            .app_data(data.clone())
            // enable logger
            .wrap(middleware::Logger::default())
            .service(pages::hello::get)
            .service(pages::hello::post)
            .service(pages::log::action)
    })
    .bind(("0.0.0.0", cli.port))?
    .run()
    .await
}
