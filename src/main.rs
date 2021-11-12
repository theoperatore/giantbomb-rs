use dotenv::dotenv;
use std::env;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let gb_token = env::var("GB_TOKEN").expect("GB_TOKEN env is required");
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to 127.0.0.1:8080");
    giantbomb_rs::srv(listener, &gb_token)?.await
}
