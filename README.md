# giantbomb-rs

A super small microservice to return a random Game from GiantBomb's Api. This is how I learn rust, doing small things over and over again lol. Who knows if I've followed html semantics right but :shrug: basically just used actix-web the whole way.

### Running the server

Clone the code, ensure you've got rust/cargo then:

```bash
GB_TOKEN=<your api key> cargo run
```

It'll install deps, compile em, then run the server on port `8080`.

### Endpoints

There's only 2 (lol)

- `/_ping` => will always return `204` as long as the server is up: Health Check
- `/games/random` => will return a random game in json.

Peep the `src/gb_client.rs` on the output schema; it's just a proxy from their api.

# License

MIT
