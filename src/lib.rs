mod gb_client;

use actix_web::dev::Server;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

struct AppContext {
  gb_token: String,
}

#[derive(Serialize)]
struct ErrorResponse {
  message: String,
}

async fn random_game(ctx: web::Data<AppContext>) -> impl Responder {
  let token = &ctx.gb_token;
  match gb_client::get_random_game(token).await {
    Ok(game) => HttpResponse::Ok().json(game),
    Err(err) => {
      tracing::error!("Error fetching game: {}", err);
      HttpResponse::BadGateway().json(ErrorResponse {
        message: "Failed to get random game".to_string(),
      })
    }
  }
}

#[tracing::instrument(name = "Ping handler", skip(_req))]
async fn ping(_req: HttpRequest) -> impl Responder {
  HttpResponse::Ok()
}

pub fn srv(listener: TcpListener, gb_token: &str) -> Result<Server, std::io::Error> {
  // since we want a string, we put the borrow into a Box on the heap
  // and move that into the closure;
  let token = Box::new(gb_token.to_string());
  let srv = HttpServer::new(move || {
    App::new()
      .wrap(TracingLogger::default())
      .app_data(web::Data::new(AppContext {
        gb_token: token.to_string(),
      }))
      .route("/_ping", web::get().to(ping))
      .route("/games/random", web::get().to(random_game))
  })
  .listen(listener)?
  .run();

  Ok(srv)
}
