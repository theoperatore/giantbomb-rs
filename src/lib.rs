mod gb_client;

use actix_web::dev::Server;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

struct AppContext {
    gb_token: String,
}

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
}

#[derive(Serialize)]
struct GameResponse {
    game: Option<gb_client::Game>,
    message: String,
}

#[derive(Serialize)]
struct SearchResponse {
    games: Vec<gb_client::Game>,
    message: String,
}

async fn random_game(ctx: web::Data<AppContext>) -> impl Responder {
    let token = &ctx.gb_token;
    match gb_client::get_random_game(token).await {
        Ok(game) => HttpResponse::Ok().json(GameResponse {
            game: Some(game),
            message: "OK".to_string(),
        }),
        Err(err) => {
            tracing::error!("Error fetching game: {}", err);
            HttpResponse::BadGateway().json(GameResponse {
                game: None,
                message: "Failed to get random game".to_string(),
            })
        }
    }
}

async fn search_game_name(
    params: web::Query<SearchRequest>,
    ctx: web::Data<AppContext>,
) -> impl Responder {
    let token = &ctx.gb_token;
    let search_term = &params.query;
    tracing::info!("searching for game with name: {}", search_term);
    match gb_client::search_by_game_name(token, search_term).await {
        Ok(results) => HttpResponse::Ok().json(SearchResponse {
            games: results,
            message: "OK".to_string(),
        }),
        Err(err) => {
            tracing::error!("Error searching for game: {}", err);
            HttpResponse::BadGateway().json(GameResponse {
                game: None,
                message: "Failed to search for game".to_string(),
            })
        }
    }
}

#[tracing::instrument(name = "Ping handler", skip(_req))]
async fn ping(_req: HttpRequest) -> impl Responder {
    HttpResponse::NoContent()
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
            .route("/search", web::get().to(search_game_name))
    })
    .listen(listener)?
    .run();

    Ok(srv)
}
