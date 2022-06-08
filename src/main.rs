use crate::starknet_client::StarkNetChain;

#[macro_use]
extern crate rocket;

mod github;
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

    let chain = std::env::var("STARKNET_CHAIN")
        .expect("STARKNET_CHAIN environment variable must be set to either 'MAINNET' or 'TESTNET'");
    let chain: StarkNetChain = chain
        .parse()
        .expect("STARKNET_CHAIN environment variable must be set to either 'MAINNET' or 'TESTNET'");

    info!("configuration loaded");

    let github_client =
        github::GitHubClient::new(github_id, github_secret, access_token_url, user_api_url);

    let starknet_client = starknet_client::StarkNetClient::new(chain);

    rocket::build()
        .manage(github_client)
        .manage(starknet_client)
        .mount("/registrations", routes![rest::register_github_user])
}
