use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::Instrument;

// required by GiantBomb otherwise the api fails with: Bad Content type
const USER_AGENT: &str = "alorg-game-of-the-day-giantbomb";

#[derive(Deserialize, Serialize, Debug)]
struct GameImage {
  original_url: Option<String>,
  super_url: Option<String>,
  screen_url: Option<String>,
  screen_large_url: Option<String>,
  medium_url: Option<String>,
  small_url: Option<String>,
  thumb_url: Option<String>,
  icon_url: Option<String>,
  tiny_url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Characteristic {
  api_detail_url: String,
  id: i32,
  name: String,
  site_detail_url: String,
  abbreviation: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Game {
  id: i32,
  guid: String,
  image: Option<GameImage>,
  name: String,
  deck: Option<String>,
  description: Option<String>,
  original_release_date: Option<String>,
  site_detail_url: Option<String>,
  expected_release_day: Option<i32>,
  expected_release_month: Option<i32>,
  expected_release_year: Option<i32>,
  expected_release_quarter: Option<i32>,
  platforms: Option<Vec<Characteristic>>,
  concepts: Option<Vec<Characteristic>>,
  developers: Option<Vec<Characteristic>>,
  characters: Option<Vec<Characteristic>>,
  themes: Option<Vec<Characteristic>>,
}

#[derive(Deserialize, Debug)]
struct DetailUrl {
  api_detail_url: String,
}

#[derive(Deserialize, Debug)]
struct GiantBombResponse {
  // I can't remember what exactly these values can be, but just someting that isn't OK will be enough
  // will be either "OK" | "ERROR"
  error: String,
  version: String,
  limit: i32,
  offset: i32,
  number_of_page_results: i64,
  number_of_total_results: i64,
  status_code: i32,
  results: Vec<DetailUrl>,
}

#[derive(Deserialize, Debug)]
struct GiantBombGameResponse {
  // I can't remember what exactly these values can be, but just someting that isn't OK will be enough
  // will be either "OK" | "ERROR"
  error: String,
  version: String,
  limit: i32,
  offset: i32,
  number_of_page_results: i64,
  number_of_total_results: i64,
  status_code: i32,
  results: Game,
}

#[derive(Deserialize, Debug)]
struct DatasResponse {
  status: String,
  result: Game,
}

fn random(max: i64) -> i64 {
  // get random int between 0 and (max - 1)
  rand::thread_rng().gen_range(0..max)
}

#[tracing::instrument(name = "Max games query", skip(client, token))]
async fn get_max_games(client: &Client, token: &str) -> Result<i64, reqwest::Error> {
  let url = format!(
    "https://www.giantbomb.com/api/games/?api_key={}&limit=1&field_list=api_detail_url&format=json",
    token
  );
  let response = client
    .get(url)
    .header("User-Agent", USER_AGENT)
    .send()
    .await?
    .json::<GiantBombResponse>()
    .await
    .map_err(|e| reqwest::Error::from(e))?;

  Ok(response.number_of_total_results)
}

#[tracing::instrument(name = "Game uri query", skip(client, token, idx), fields(game_idx = %idx))]
async fn get_game_uri(client: &Client, token: &str, idx: i64) -> Result<String, reqwest::Error> {
  let url = format!(
    "https://www.giantbomb.com/api/games/?api_key={}&limit=1&format=json&offset={}&field_list=api_detail_url",
    token, idx
  );

  let response = client
    .get(url)
    .header("User-Agent", USER_AGENT)
    .send()
    .await?
    .json::<GiantBombResponse>()
    .await?;

  let url = response.results.get(0).map(|detail| &detail.api_detail_url);
  match url {
    None => Ok("".to_string()),
    Some(url) => Ok(url.to_string()),
  }
}

#[tracing::instrument(name = "Game details query", skip(client, token, uri), fields(giantbomb_uri = %uri))]
async fn get_game_details(client: &Client, token: &str, uri: &str) -> Result<Game, reqwest::Error> {
  let url = format!(
    "{}?api_key={}&format=json&field_list={}",
    uri,
    token,
    [
      "name",
      "site_detail_url",
      "themes",
      "platforms",
      "original_release_date",
      "image",
      "id",
      "guid",
      "expected_release_year",
      "expected_release_quarter",
      "expected_release_month",
      "expected_release_day",
      "developers",
      "deck",
      // "description", // mostly html formatted nonsense that sometimes is huge in bytes
      "concepts",
      "characters",
    ]
    .join(",")
  );

  let json_span = tracing::info_span!("Game detail deserialization");
  let response = client
    .get(url)
    .header("User-Agent", USER_AGENT)
    .send()
    .await?
    .json::<GiantBombGameResponse>()
    .instrument(json_span)
    .await
    .map_err(|e| {
      tracing::error!("Error fetching game_details: {:?}", e);
      reqwest::Error::from(e)
    })?;

  Ok(response.results)
}

#[tracing::instrument(name = "Get random game", skip(token))]
pub async fn get_random_game(token: &str) -> Result<Game, reqwest::Error> {
  let client = reqwest::Client::new();
  let max_games = get_max_games(&client, token).await?;

  let idx = random(max_games);

  // this game uri has a HUGE detail payload
  // let game_uri = "https://www.giantbomb.com/api/game/3030-1156/";
  let game_uri = get_game_uri(&client, token, idx).await?;

  let game = get_game_details(&client, token, &game_uri).await?;
  Ok(game)
}
