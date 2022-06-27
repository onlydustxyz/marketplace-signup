use crate::{
    contracts::badge_registry::{self, BadgeRegistryClient},
    identity_providers::IdentityProvider,
};
use rocket::{
    http::Status,
    serde::{json::Json, Deserialize},
    State,
};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet::core::types::FieldElement;

#[serde_as]
#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[serde(crate = "rocket::serde")]
pub struct Signature {
    #[serde_as(as = "UfeHex")]
    pub r: FieldElement,
    #[serde_as(as = "UfeHex")]
    pub s: FieldElement,
}

#[serde_as]
#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[serde(crate = "rocket::serde")]
pub struct SignedData {
    #[serde_as(as = "UfeHex")]
    hash: FieldElement,
    signature: Signature,
}

impl From<SignedData> for badge_registry::SignedData {
    fn from(data: SignedData) -> Self {
        Self {
            hash: data.hash,
            signature: badge_registry::Signature {
                r: data.signature.r,
                s: data.signature.s,
            },
        }
    }
}

#[serde_as]
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GithubUserRegistrationRequest<'r> {
    authorization_code: &'r str,
    #[serde_as(as = "UfeHex")]
    account_address: FieldElement,
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

    let result = badge_registry_client
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

    info!(
        "successfully registered user with GitHub ID {} and StarkNet account {}",
        user_id, registration.account_address
    );
    Status::NoContent
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use rocket::{
        http::{ContentType, Status},
        local::blocking::Client,
        serde::json::serde_json::json,
    };
    use starknet::macros::felt;

    use crate::{
        contracts::badge_registry::{BadgeRegistryClient, MockBadgeRegistryClient},
        identity_providers::{IdentityProvider, MockIdentityProvider},
        rest,
    };

    #[test]
    fn test_register_github_user() {
        let mut github_mock = MockIdentityProvider::new();

        github_mock
            .expect_new_access_token()
            .with(eq("foo-code"))
            .times(1)
            .returning(|_| Ok("foo-token".to_string()));

        github_mock
            .expect_get_user_id()
            .with(eq("foo-token"))
            .times(1)
            .returning(|_| Ok(42));

        let mut badge_registry_mock = MockBadgeRegistryClient::new();

        badge_registry_mock
            .expect_check_signature()
            .with(
                eq(crate::contracts::badge_registry::SignedData {
                    hash: felt!(
                        "0x287b943b1934949486006ad63ac0293038b6c818b858b09f8e0a9da12fc4074"
                    ),
                    signature: crate::contracts::badge_registry::Signature {
                        r: felt!(
                            "0xde4d49b21dd8714eaf5a1b480d8ede84d2230d1763cfe06762d8a117493bcd"
                        ),
                        s: felt!(
                            "0x4b61402b98b29a34bd4cba8b5eabae840809914160002385444059f59449a4"
                        ),
                    },
                }),
                eq(felt!(
                    "0x65f1506b7f974a1355aeebc1314579326c84a029cd8257a91f82384a6a0ace"
                )),
            )
            .times(1)
            .returning(|_, _| Ok(()));

        badge_registry_mock
            .expect_register_user()
            .with(
                eq(felt!(
                    "0x65f1506b7f974a1355aeebc1314579326c84a029cd8257a91f82384a6a0ace"
                )),
                eq(42),
            )
            .times(1)
            .returning(|_, _| Ok(()));

        let router = rest::router::new(
            Box::new(github_mock) as Box<dyn IdentityProvider>,
            Box::new(badge_registry_mock) as Box<dyn BadgeRegistryClient>,
        );

        let client = Client::tracked(router).expect("valid rocket instance");
        let response = client
            .post(uri!("/registrations/github"))
            .header(ContentType::JSON)
            .body(
                json!({
                    "authorization_code": "foo-code",
                    "account_address": "0x65f1506b7f974a1355aeebc1314579326c84a029cd8257a91f82384a6a0ace",
                    "signed_data": {
                        "hash": "0x287b943b1934949486006ad63ac0293038b6c818b858b09f8e0a9da12fc4074",
                        "signature": {
                            "r": "0xde4d49b21dd8714eaf5a1b480d8ede84d2230d1763cfe06762d8a117493bcd",
                            "s": "0x4b61402b98b29a34bd4cba8b5eabae840809914160002385444059f59449a4"
                        }
                    },
                })
                .to_string(),
            )
            .dispatch();

        assert_eq!(response.status(), Status::NoContent);
    }
}
