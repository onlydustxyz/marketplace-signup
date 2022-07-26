use dotenv::dotenv;

use crate::{
    application::registerer::{Registerer, RegistererImpl},
    infrastructure::{github_client::GitHubClient, starknet_client::StarkNetClient},
};

#[macro_use]
extern crate rocket;

mod application;
mod config;
mod domain;
mod infrastructure;
mod rest;

#[launch]
fn rocket() -> _ {
    info!("loading configuration...");
    dotenv().ok();
    let conf = config::load();
    info!("configuration loaded");

    let github_client = GitHubClient::new(
        conf.github_id,
        conf.github_secret,
        conf.access_token_url,
        conf.user_api_url,
    );
    let starknet_client = StarkNetClient::new(
        &conf.hex_account_address,
        &conf.hex_private_key,
        &conf.hex_badge_registry_address,
        conf.chain,
    );
    let registerer = RegistererImpl::new(github_client, starknet_client);

    rest::router::new(Box::new(registerer) as Box<dyn Registerer<GitHubClient, StarkNetClient>>)
}
