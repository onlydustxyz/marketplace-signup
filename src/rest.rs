use crate::github;
use rocket::{
    http::Status,
    serde::{json::Json, Deserialize},
    State,
};
use starknet::core::types::FieldElement;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GithubUserRegistrationRequest<'r> {
    authorization_code: &'r str,
    signed_authorization_code: &'r str,
    account_address: FieldElement,
}

#[post("/github", format = "json", data = "<registration>")]
pub async fn register_github_user(
    registration: Json<GithubUserRegistrationRequest<'_>>,
    github_client: &State<github::GitHubClient>,
) -> Status {
    let access_token = github_client
        .new_access_token(registration.authorization_code)
        .await;

    let access_token = match access_token {
        Ok(access_token) => access_token,
        Err(e) => {
            warn!(
                "failed to get new GitHub access token from code {}. Error: {}",
                registration.authorization_code, e
            );
            return Status::Unauthorized;
        }
    };

    let user_id = github_client.get_user_id(&access_token).await;

    let user_id = match user_id {
        Ok(user_id) => user_id,
        Err(e) => {
            error!(
                "failed to get GitHub user id with access token {}. Error: {}",
                access_token, e
            );
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
