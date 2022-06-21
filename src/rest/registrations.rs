use crate::{
    contracts::badge_registry::{self, BadgeRegistryClient},
    identity_providers::IdentityProvider,
};
use rocket::{
    http::Status,
    serde::{json::Json, Deserialize},
    State,
};
use starknet::core::types::FieldElement;

use crate::rest::felt::HexFieldElement;

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[serde(crate = "rocket::serde")]
pub struct Signature {
    pub r: HexFieldElement,
    pub s: HexFieldElement,
}

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[serde(crate = "rocket::serde")]
pub struct SignedData {
    hash: HexFieldElement,
    signature: Signature,
}

impl From<SignedData> for badge_registry::SignedData {
    fn from(data: SignedData) -> Self {
        Self {
            hash: FieldElement::from(data.hash),
            signature: badge_registry::Signature {
                r: FieldElement::from(data.signature.r),
                s: FieldElement::from(data.signature.s),
            },
        }
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GithubUserRegistrationRequest<'r> {
    authorization_code: &'r str,
    account_address: HexFieldElement,
    signed_data: SignedData,
}

#[post("/github", format = "json", data = "<registration>")]
pub async fn register_github_user(
    registration: Json<GithubUserRegistrationRequest<'_>>,
    github_identity_provider: &State<Box<dyn IdentityProvider>>,
    badge_registry_client: &State<Box<dyn BadgeRegistryClient>>,
) -> Status {
    let access_token = github_identity_provider
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

    let user_id = github_identity_provider.get_user_id(&access_token).await;

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

    let result = badge_registry_client
        .check_signature(
            badge_registry::SignedData::from(registration.signed_data),
            FieldElement::from(registration.account_address),
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

    let result = badge_registry_client
        .register_user(FieldElement::from(registration.account_address), user_id)
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

    info!(
        "successfully registered user with GitHub ID {} and StarkNet account {}",
        user_id, registration.account_address
    );
    Status::NoContent
}
