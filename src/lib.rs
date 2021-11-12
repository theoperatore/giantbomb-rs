use actix_web::dev::Server;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;

async fn ping(_req: HttpRequest) -> impl Responder {
  HttpResponse::Ok()
}

pub fn srv(listener: TcpListener, _gb_token: &str) -> Result<Server, std::io::Error> {
  let srv = HttpServer::new(|| App::new().route("/_ping", web::get().to(ping)))
    .listen(listener)?
    .run();

  Ok(srv)
}
