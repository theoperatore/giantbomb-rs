use std::net::TcpListener;

#[actix_rt::test]
async fn ping_returns_200_ok() {
  // Arrange
  let addr = spawn_app();
  let client = reqwest::Client::new();

  // Act
  let response = client
    .get(format!("{}/_ping", addr))
    .send()
    .await
    .expect("Failed to execute request.");

  // Assert
  assert!(response.status().is_success());
  assert_eq!(Some(0), response.content_length());
}

// Launch our application in the background with any open port
fn spawn_app() -> String {
  let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
  let port = listener.local_addr().unwrap().port();

  let server = giantbomb_rs::srv(listener).expect("Failed to create server");
  let _ = tokio::spawn(server);

  format!("http://127.0.0.1:{}", port)
}
