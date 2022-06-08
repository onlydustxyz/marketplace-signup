#[macro_use]
extern crate rocket;

mod github;
mod rest;

#[launch]
fn rocket() -> _ {
    info!("loading configuration...");

    let github_id = std::env::var("GITHUB_ID").unwrap();
    let github_secret = std::env::var("GITHUB_SECRET").unwrap();
    let access_token_url = std::env::var("GITHUB_ACCESS_TOKEN_URL")
        .unwrap_or_else(|_| "https://github.com/login/oauth/access_token".to_string());
    let user_api_url = std::env::var("GITHUB_USER_API_URL")
        .unwrap_or_else(|_| "https://api.github.com/user".to_string());

    info!("configuration loaded");

    let github_client =
        github::GitHubClient::new(github_id, github_secret, access_token_url, user_api_url);

    rocket::build()
        .manage(github_client)
        .mount("/registrations", routes![rest::register_github_user])
}
