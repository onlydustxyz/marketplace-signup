use contracts::badge_registry::BadgeRegistryClient;
use dotenv::dotenv;
use identity_providers::{github, IdentityProvider};
use opentelemetry::global;

#[macro_use]
extern crate rocket;

mod config;
mod contracts;
mod identity_providers;
mod rest;

const SERVICE_NAME: &str = "od-badge-signup";

#[rocket::main]
async fn main() {
    info!("loading configuration...");
    dotenv().ok();
    let conf = config::load();
    info!("configuration loaded");

    info!("starting Open Telemetry");
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    let _ = opentelemetry_jaeger::new_pipeline()
        .with_service_name(SERVICE_NAME)
        //.install_batch(opentelemetry::runtime::Tokio)
        .install_simple()
        .unwrap();

    let github_client = github::GitHubClient::new(
        conf.github_id,
        conf.github_secret,
        conf.access_token_url,
        conf.user_api_url,
    );
    let starknet_client = contracts::client::StarkNetClient::new(
        &conf.hex_account_address,
        &conf.hex_private_key,
        &conf.hex_badge_registry_address,
        conf.chain,
    );

    let _ = rest::router::new(
        Box::new(github_client) as Box<dyn IdentityProvider>,
        Box::new(starknet_client) as Box<dyn BadgeRegistryClient>,
    )
    .launch()
    .await;

    global::shutdown_tracer_provider();
}
