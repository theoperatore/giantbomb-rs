use rand::Rng;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::TracingMiddleware;
use serde::{Deserialize, Serialize};

type Error = Box<dyn std::error::Error + Send + Sync>;

// required by GiantBomb otherwise the api fails with: Bad Content type
const USER_AGENT: &str = "alorg-game-of-the-day-giantbomb";

#[derive(Deserialize, Serialize, Debug, PartialEq)]
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

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Characteristic {
    api_detail_url: String,
    id: i32,
    name: String,
    site_detail_url: String,
    abbreviation: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
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

#[derive(Deserialize, Serialize, Debug)]
struct DetailUrl {
    api_detail_url: String,
}

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
struct GiantBombSearchResponse {
    // I can't remember what exactly these values can be, but just someting that isn't OK will be enough
    // will be either "OK" | "ERROR"
    error: String,
    version: String,
    limit: i32,
    offset: i32,
    number_of_page_results: i64,
    number_of_total_results: i64,
    status_code: i32,
    results: Vec<Game>,
}

fn random(max: i64) -> i64 {
    // get random int between 0 and (max - 1)
    rand::thread_rng().gen_range(0..max)
}

#[tracing::instrument(name = "Max games query", skip(client, token))]
async fn get_max_games(
    client: &ClientWithMiddleware,
    token: &str,
    api: &str,
) -> Result<i64, Error> {
    let url = format!(
        "{}/api/games/?api_key={}&limit=1&format=json&field_list=api_detail_url",
        api, token
    );
    let response = client
        .get(url)
        .send()
        .await
        .and_then(|r| match r.error_for_status() {
            Ok(res) => Ok(res),
            Err(err) => Err(reqwest_middleware::Error::Reqwest(err)),
        })?
        .json::<GiantBombResponse>()
        .await?;

    Ok(response.number_of_total_results)
}

#[tracing::instrument(name = "Game uri query", skip(client, token, idx), fields(game_idx = %idx))]
async fn get_game_uri(
    client: &ClientWithMiddleware,
    token: &str,
    idx: i64,
    api: &str,
) -> Result<String, Error> {
    let url = format!(
        "{}/api/games/?api_key={}&limit=1&format=json&offset={}&field_list=api_detail_url",
        api, token, idx
    );

    let response = client
        .get(url)
        .send()
        .await
        .and_then(|r| match r.error_for_status() {
            Ok(res) => Ok(res),
            Err(err) => Err(reqwest_middleware::Error::Reqwest(err)),
        })?
        .json::<GiantBombResponse>()
        .await?;

    let url = response.results.get(0).map(|detail| &detail.api_detail_url);
    match url {
        None => Ok("".to_string()),
        Some(url) => Ok(url.to_string()),
    }
}

#[tracing::instrument(name = "Game details query", skip(client, token, uri), fields(giantbomb_uri = %uri))]
async fn get_game_details(
    client: &ClientWithMiddleware,
    token: &str,
    uri: &str,
) -> Result<Game, Error> {
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

    let response = client
        .get(url)
        .send()
        .await
        .and_then(|r| match r.error_for_status() {
            Ok(res) => Ok(res),
            Err(err) => Err(reqwest_middleware::Error::Reqwest(err)),
        })?
        .json::<GiantBombGameResponse>()
        .await?;

    Ok(response.results)
}

#[tracing::instrument(name = "Get random game", skip(token))]
pub async fn get_random_game(token: &str) -> Result<Game, Error> {
    fetch_game(token, "https://www.giantbomb.com").await
}

pub async fn search_by_game_name(token: &str, search_term: &str) -> Result<Vec<Game>, Error> {
    let client = ClientBuilder::new(
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("failed to build reqwest client"),
    )
    .with(TracingMiddleware)
    .build();

    let url  =  format!("https://www.giantbomb.com/api/search?api_key={}&limit=5&format=json&resources=game&field_list={}&query={}", token, [
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
        ].join(","), search_term);

    let response = client
        .get(url)
        .send()
        .await
        .and_then(|r| match r.error_for_status() {
            Ok(res) => Ok(res),
            Err(err) => Err(reqwest_middleware::Error::Reqwest(err)),
        })?
        .json::<GiantBombSearchResponse>()
        .await?;

    Ok(response.results)
}

// so that I can test the client with a mock uri
async fn fetch_game(token: &str, gb_api_url: &str) -> Result<Game, Error> {
    let client = ClientBuilder::new(
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("failed to build reqwest client"),
    )
    .with(TracingMiddleware)
    .build();

    let max_games = get_max_games(&client, token, gb_api_url).await?;

    let idx = random(max_games);

    // this game uri has a HUGE detail payload
    // let game_uri = "https://www.giantbomb.com/api/game/3030-1156/";
    let game_uri = get_game_uri(&client, token, idx, gb_api_url).await?;

    let game = get_game_details(&client, token, &game_uri).await?;
    Ok(game)
}

#[cfg(test)]
mod tests {
    use crate::gb_client::fetch_game;
    use crate::gb_client::DetailUrl;
    use crate::gb_client::Game;
    use crate::gb_client::GiantBombGameResponse;
    use crate::gb_client::GiantBombResponse;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::ResponseTemplate;
    use wiremock::{Mock, MockServer};

    #[tokio::test]
    async fn throws_error_when_max_games_returns_non_ok() {
        // Arrange
        let mock_gb_server = MockServer::start().await;
        let _mock_guard = Mock::given(method("GET"))
            .and(path("/api/games/"))
            .respond_with(ResponseTemplate::new(500))
            .named("GET max games")
            .expect(1)
            .mount_as_scoped(&mock_gb_server)
            .await;

        // Act
        let result = fetch_game("fake_token", &mock_gb_server.uri()).await;

        // Assert
        assert_eq!(result.is_err(), true);
    }

    #[tokio::test]
    async fn throws_error_when_game_uri_returns_non_ok() {
        // Arrange
        let mock_gb_server = MockServer::start().await;
        let max_games_response = GiantBombResponse {
            error: String::from("OK"),
            version: String::from("1"),
            limit: 1,
            offset: 0,
            number_of_page_results: 1,
            number_of_total_results: 1,
            status_code: 200,
            results: Vec::new(),
        };
        let _mock_guard = Mock::given(method("GET"))
            .and(path("/api/games/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(max_games_response))
            .named("GET max games")
            .expect(1)
            .up_to_n_times(1)
            .mount_as_scoped(&mock_gb_server)
            .await;

        let _mock_guard_2 = Mock::given(method("GET"))
            .and(path("/api/games/"))
            .and(query_param("limit", "1"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(500))
            .named("GET games uri")
            .expect(1)
            .up_to_n_times(1)
            .mount_as_scoped(&mock_gb_server)
            .await;

        // Act
        let result = fetch_game("fake_token", &mock_gb_server.uri()).await;

        // Assert
        assert_eq!(result.is_err(), true);
    }

    #[tokio::test]
    async fn throws_error_when_game_details_returns_non_ok() {
        // Arrange
        let mock_gb_server = MockServer::start().await;
        let max_games_response = GiantBombResponse {
            error: String::from("OK"),
            version: String::from("1"),
            limit: 1,
            offset: 0,
            number_of_page_results: 1,
            number_of_total_results: 1,
            status_code: 200,
            results: Vec::new(),
        };

        let game_uri_response = GiantBombResponse {
            error: String::from("OK"),
            version: String::from("1"),
            limit: 1,
            offset: 0,
            number_of_page_results: 1,
            number_of_total_results: 1,
            status_code: 200,
            results: vec![DetailUrl {
                api_detail_url: format!("{}/api/game/123", mock_gb_server.uri()),
            }],
        };

        let _mock_guard = Mock::given(method("GET"))
            .and(path("/api/games/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(max_games_response))
            .named("GET max games")
            .expect(1)
            .up_to_n_times(1)
            .mount_as_scoped(&mock_gb_server)
            .await;

        let _mock_guard_2 = Mock::given(method("GET"))
            .and(path("/api/games/"))
            .and(query_param("limit", "1"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(game_uri_response))
            .named("GET games uri")
            .expect(1)
            .up_to_n_times(1)
            .mount_as_scoped(&mock_gb_server)
            .await;

        let _mock_guard_3 = Mock::given(method("GET"))
            .and(path("/api/game/123"))
            .respond_with(ResponseTemplate::new(500))
            .named("GET game details")
            .expect(1)
            .up_to_n_times(1)
            .mount_as_scoped(&mock_gb_server)
            .await;

        // Act
        let result = fetch_game("fake_token", &mock_gb_server.uri()).await;

        // Assert
        assert_eq!(result.is_err(), true);
    }

    #[tokio::test]
    async fn returns_game_details() {
        // Arrange
        let mock_gb_server = MockServer::start().await;
        let max_games_response = GiantBombResponse {
            error: String::from("OK"),
            version: String::from("1"),
            limit: 1,
            offset: 0,
            number_of_page_results: 1,
            number_of_total_results: 1,
            status_code: 200,
            results: Vec::new(),
        };

        let game_uri_response = GiantBombResponse {
            error: String::from("OK"),
            version: String::from("1"),
            limit: 1,
            offset: 0,
            number_of_page_results: 1,
            number_of_total_results: 1,
            status_code: 200,
            results: vec![DetailUrl {
                api_detail_url: format!("{}/api/game/123", mock_gb_server.uri()),
            }],
        };

        let game_response = GiantBombGameResponse {
            error: String::from("OK"),
            version: String::from("1"),
            limit: 1,
            offset: 0,
            number_of_page_results: 1,
            number_of_total_results: 1,
            status_code: 200,
            results: Game::default(),
        };

        let _mock_guard = Mock::given(method("GET"))
            .and(path("/api/games/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(max_games_response))
            .named("GET max games")
            .expect(1)
            .up_to_n_times(1)
            .mount_as_scoped(&mock_gb_server)
            .await;

        let _mock_guard_2 = Mock::given(method("GET"))
            .and(path("/api/games/"))
            .and(query_param("limit", "1"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(game_uri_response))
            .named("GET games uri")
            .expect(1)
            .up_to_n_times(1)
            .mount_as_scoped(&mock_gb_server)
            .await;

        let _mock_guard_3 = Mock::given(method("GET"))
            .and(path("/api/game/123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(game_response))
            .named("GET game details")
            .expect(1)
            .up_to_n_times(1)
            .mount_as_scoped(&mock_gb_server)
            .await;

        // Act
        let result = fetch_game("fake_token", &mock_gb_server.uri()).await;

        // Assert
        assert_eq!(result.is_err(), false);
        assert_eq!(result.unwrap(), Game::default());
    }
}
