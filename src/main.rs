use dotenv::dotenv;
use opentelemetry::{global, sdk::propagation::TraceContextPropagator};
use std::env;
use std::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

fn addr() -> &'static str {
    match env::var("RUST_ENV").unwrap_or_default().as_ref() {
        "production" => "0.0.0:8080",
        _ => "127.0.0.1:8080",
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let app_name = "giantbomb-rs";
    tracing_log::LogTracer::init().expect("failed to init logger");
    global::set_text_map_propagator(TraceContextPropagator::new());

    // let tracer = opentelemetry_jaeger::new_pipeline()
    //     .with_service_name(app_name)
    //     .install_batch(TokioCurrentThread)
    //     .expect("Failed to install OpenTelemetry tracer.");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let formatting_layer = BunyanFormattingLayer::new(app_name.to_string(), std::io::stdout);
    // let formatting_layer = fmt::Layer::default();

    let subscriber = Registry::default()
        .with(env_filter)
        // .with(telemetry)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set subscriber");

    let gb_token = env::var("GB_TOKEN").expect("GB_TOKEN env is required");
    let address = addr();
    let listener = TcpListener::bind(address)
        .expect(format!("Failed to bind to address: {:?}", address).as_ref());
    giantbomb_rs::srv(listener, &gb_token)?.await?;

    global::shutdown_tracer_provider();

    Ok(())
}
