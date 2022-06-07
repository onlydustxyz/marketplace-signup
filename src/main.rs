#[macro_use]
extern crate rocket;

mod github;

use rocket::{
    http::Status,
    serde::{json::Json, Deserialize},
};
use starknet::core::types::FieldElement;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/registrations", routes![register_github_user])
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct GithubUserRegistrationRequest<'r> {
    authorization_code: &'r str,
    signed_authorization_code: &'r str,
    account_address: FieldElement,
}

#[post("/github", format = "json", data = "<registration>")]
async fn register_github_user(registration: Json<GithubUserRegistrationRequest<'_>>) -> Status {
    let github_client = github::GitHubClient::new();

    // Call POST https://github.com/login/oauth/access_token with id+secret+authorization_code
    // to get an access_token (and BTW check that the authorization_code is valid).
    let access_token = github_client
        .new_access_token(registration.authorization_code)
        .await;
    let access_token = match access_token {
        Ok(access_token) => access_token,
        Err(e) => {
            //TODO: log
            return Status::Unauthorized;
        }
    };

    // Call GET https://api.github.com/user with the access_token in the Authorization header to
    // get information about the logged user, in particular, its ID.
    let user_id = github_client.get_user_id(&access_token).await;
    let user_id = match user_id {
        Ok(user_id) => user_id,
        Err(e) => {
            //TODO: log
            return Status::InternalServerError;
        }
    };

    // TODO: call `is_valid_signature` @view function on the Account smart contract (at account_address) to check that
    // the signed_authorization_code is valid and belongs to the given account.

    // TODO: call `register` @external function of the Registry smart contract, passing the github user ID and the account_address
    // as arguments.
    // This call must be done from our own account which has the right to call the `register` function of Registry smart contract.

    Status::NoContent
}
