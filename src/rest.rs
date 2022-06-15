use std::fmt;

use crate::{github, starknet_client};
use rocket::{
    http::Status,
    serde::{json::Json, Deserialize},
    State,
};
use starknet::core::types::FieldElement;

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[serde(crate = "rocket::serde")]
pub struct Signature {
    pub r: FieldElement,
    pub s: FieldElement,
}

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[serde(crate = "rocket::serde")]
pub struct SignedData {
    hash: FieldElement,
    signature: Signature,
}

impl fmt::Display for SignedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "hash={} <r={}, s={}>",
            self.hash, self.signature.r, self.signature.s
        )
    }
}

impl From<SignedData> for starknet_client::SignedData {
    fn from(data: SignedData) -> Self {
        Self {
            hash: data.hash,
            signature: starknet_client::Signature {
                r: data.signature.r,
                s: data.signature.s,
            },
        }
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GithubUserRegistrationRequest<'r> {
    authorization_code: &'r str,
    account_address: FieldElement,
    signed_data: SignedData,
}

#[post("/github", format = "json", data = "<registration>")]
pub async fn register_github_user(
    registration: Json<GithubUserRegistrationRequest<'_>>,
    github_client: &State<github::GitHubClient>,
    starknet_client: &State<starknet_client::StarkNetClient>,
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

    let result = starknet_client
        .check_signature(
            starknet_client::SignedData::from(registration.signed_data),
            registration.account_address,
        )
        .await;

    match result {
        Ok(_) => (),
        Err(e) => {
            error!(
                "signed data has an invalid signature for account {}. Error: {}",
                registration.account_address, e
            );
            return Status::Unauthorized;
        }
    }

    let result = starknet_client
        .register_user(registration.account_address, user_id)
        .await;

    match result {
        Ok(_) => (),
        Err(e) => {
            error!(
                "failed to register account {} in badge registry. Error: {}",
                registration.account_address, e
            );
            return Status::InternalServerError;
        }
    }

    Status::NoContent
}

#[derive(Responder)]
#[response(status = 200, content_type = "json")]
pub struct RawOk(&'static str);

#[get("/health")]
pub async fn health_check() -> RawOk {
    RawOk("{\"status\":\"ok\"}")
}
