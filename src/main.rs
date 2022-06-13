use crate::starknet_client::StarkNetChain;

#[macro_use]
extern crate rocket;

mod github;
mod hash_check;
mod rest;
mod starknet_client;

#[launch]
fn rocket() -> _ {
    info!("loading configuration...");

    let github_id = std::env::var("GITHUB_ID").expect("GITHUB_ID environment variable must be set");
    let github_secret =
        std::env::var("GITHUB_SECRET").expect("GITHUB_SECRET environment variable must be set");
    let access_token_url = std::env::var("GITHUB_ACCESS_TOKEN_URL")
        .unwrap_or_else(|_| "https://github.com/login/oauth/access_token".to_string());
    let user_api_url = std::env::var("GITHUB_USER_API_URL")
        .unwrap_or_else(|_| "https://api.github.com/user".to_string());

    let hex_account_address = std::env::var("STARKNET_ACCOUNT")
        .expect("STARKNET_ACCOUNT environment variable must be set");
    let hex_private_key = std::env::var("STARKNET_PRIVATE_KEY")
        .expect("STARKNET_PRIVATE_KEY environment variable must be set");
    let hex_badge_registry_address = std::env::var("STARKNET_BADGE_REGISTRY_ADDRESS")
        .expect("STARKNET_BADGE_REGISTRY_ADDRESS environment variable must be set");
    let chain = std::env::var("STARKNET_CHAIN")
        .expect("STARKNET_CHAIN environment variable must be set to either 'MAINNET' or 'TESTNET'");
    let chain: StarkNetChain = chain
        .parse()
        .expect("STARKNET_CHAIN environment variable must be set to either 'MAINNET' or 'TESTNET'");

    info!("configuration loaded");

    let github_client =
        github::GitHubClient::new(github_id, github_secret, access_token_url, user_api_url);

    let starknet_client = starknet_client::StarkNetClient::new(
        &hex_account_address,
        &hex_private_key,
        &hex_badge_registry_address,
        chain,
    );

    rocket::build()
        .manage(github_client)
        .manage(starknet_client)
        .mount("/", routes![rest::health_check])
        .mount("/registrations", routes![rest::register_github_user])
}
