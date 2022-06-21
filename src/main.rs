use contracts::badge_registry::BadgeRegistryClient;
use identity_providers::{github, IdentityProvider};

#[macro_use]
extern crate rocket;

mod config;
mod contracts;
mod identity_providers;
mod rest;

#[launch]
fn rocket() -> _ {
    info!("loading configuration...");
    let conf = config::load();
    info!("configuration loaded");

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

    rest::router::new(
        Box::new(github_client) as Box<dyn IdentityProvider>,
        Box::new(starknet_client) as Box<dyn BadgeRegistryClient>,
    )
}
