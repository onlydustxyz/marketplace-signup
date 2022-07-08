use crate::{
    contracts::badge_registry::{self, BadgeRegistryClient},
    identity_providers::IdentityProvider,
    rest::problem,
    rest::problem::ProblemResponse,
};
use rocket::{
    http::Status,
    serde::{json::Json, Deserialize, Serialize},
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

#[serde_as]
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GithubUserRegistrationResponse {
    #[serde_as(as = "UfeHex")]
    transaction_hash: FieldElement,
}

#[post("/github", format = "json", data = "<registration>")]
pub async fn register_github_user(
    registration: Json<GithubUserRegistrationRequest<'_>>,
    github_identity_provider: &State<Box<dyn IdentityProvider>>,
    badge_registry_client: &State<Box<dyn BadgeRegistryClient>>,
) -> Result<Json<GithubUserRegistrationResponse>, ProblemResponse> {
    let access_token = github_identity_provider
        .new_access_token(registration.authorization_code)
        .await;

    let access_token = match access_token {
        Ok(access_token) => access_token,
        Err(e) => {
            warn!(
                "Failed to get new GitHub access token from code {}. Error: {}",
                registration.authorization_code, e
            );
            return Err(problem::new_response(
                Status::Unauthorized,
                "Invalid GitHub code",
                format!(
                    "Failed to get new GitHub access token from code {}",
                    registration.authorization_code
                ),
            ));
        }
    };

    let user_id = github_identity_provider.get_user_id(&access_token).await;

    let user_id = match user_id {
        Ok(user_id) => user_id,
        Err(e) => {
            error!(
                "Failed to get GitHub user id with access token {}. Error: {}",
                access_token, e
            );
            return Err(problem::new_response(
                Status::InternalServerError,
                "GitHub GET /user failure",
                format!(
                    "Failed to get GitHub user id with access token {}",
                    access_token
                ),
            ));
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
            warn!(
                "Signed data has an invalid signature for account {}. Error: {}",
                registration.account_address, e
            );
            return Err(problem::new_response(
                Status::InternalServerError,
                "Invalid signature",
                format!(
                    "Signed data has an invalid signature for account {}",
                    registration.account_address
                ),
            ));
        }
    }

    let result = badge_registry_client
        .register_user(registration.account_address, user_id)
        .await;

    let transaction_result = match result {
        Ok(transaction_result) => transaction_result,
        Err(e) => {
            error!(
                "Failed to register account {} in the registry contract. Error: {}",
                registration.account_address, e
            );
            return Err(problem::new_response(
                Status::InternalServerError,
                "Transaction error",
                format!(
                    "Failed to register account {} in the registry contract",
                    registration.account_address
                ),
            ));
        }
    };

    info!(
        "successfully registered user with GitHub ID {} and StarkNet account {}",
        user_id, registration.account_address
    );
    Ok(Json(GithubUserRegistrationResponse {
        transaction_hash: transaction_result.transaction_hash,
    }))
}

#[cfg(test)]
mod tests {
    use claim::assert_some_eq;
    use mockall::predicate::eq;
    use rocket::{
        http::{ContentType, Status},
        local::blocking::Client,
        serde::json::serde_json::json,
    };
    use starknet::{core::types::AddTransactionResult, macros::felt};

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
            .returning(|_, _| {
                Ok(AddTransactionResult {
                    code: starknet::core::types::AddTransactionResultCode::TransactionReceived,
                    transaction_hash: felt!("0x666"),
                    address: None,
                    class_hash: None,
                })
            });

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

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string();
        assert_some_eq!(body, "{\"transaction_hash\":\"0x666\"}".to_string());
    }
}
