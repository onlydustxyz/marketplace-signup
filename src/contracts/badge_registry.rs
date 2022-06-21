use starknet::{
    accounts::{single_owner::TransactionError, Account, Call},
    core::{
        types::{BlockId, FieldElement, InvokeFunctionTransactionRequest},
        utils::get_selector_from_name,
    },
    providers::Provider,
};

use super::{client::StarkNetClient, errors::StarknetError};

#[rocket::async_trait]
pub trait BadgeRegistryClient: Send + Sync {
    async fn check_signature(
        &self,
        signed_data: SignedData,
        account_address: FieldElement,
    ) -> Result<(), StarknetError>;

    async fn register_user(
        &self,
        user_account_address: FieldElement,
        github_user_id: u64,
    ) -> Result<(), StarknetError>;
}

/// Stark ECDSA signature
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Signature {
    /// The `r` value of a signature
    pub r: FieldElement,
    /// The `s` value of a signature
    pub s: FieldElement,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct SignedData {
    pub hash: FieldElement,
    pub signature: Signature,
}

#[rocket::async_trait]
impl BadgeRegistryClient for StarkNetClient {
    async fn check_signature(
        &self,
        signed_data: SignedData,
        account_address: FieldElement,
    ) -> Result<(), StarknetError> {
        self.provider
            .call_contract(
                InvokeFunctionTransactionRequest {
                    contract_address: account_address,
                    entry_point_selector: get_selector_from_name("is_valid_signature").unwrap(),
                    calldata: vec![
                        signed_data.hash,
                        FieldElement::from(2u64),
                        signed_data.signature.r,
                        signed_data.signature.s,
                    ],
                    signature: vec![],
                    max_fee: FieldElement::ZERO,
                },
                BlockId::Latest,
            )
            .await
            .map_err(TransactionError::ProviderError)
            .map_err(StarknetError::TransactionError)?;

        Ok(())
    }

    async fn register_user(
        &self,
        user_account_address: FieldElement,
        github_user_id: u64,
    ) -> Result<(), StarknetError> {
        self.account
            .execute(&[Call {
                to: self.badge_registry_address,
                selector: get_selector_from_name("register_github_handle").unwrap(),
                calldata: vec![user_account_address, FieldElement::from(github_user_id)],
            }])
            .send()
            .await
            .map_err(StarknetError::TransactionError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::contracts::badge_registry::{BadgeRegistryClient, Signature};
    use crate::contracts::client::StarkNetChain;
    use crate::contracts::{self, client::StarkNetClient};

    use rocket::tokio;
    use starknet::core::types::FieldElement;
    use std::time::{SystemTime, UNIX_EPOCH};

    const ADMIN_TEST_ACCOUNT: &str =
        "0x7343772b33dd34cbb1e23b9abefdde5b7addccb3e3c66943b78e5e52d416c29";
    const ADMIN_TEST_PRIVATE_KEY: &str =
        "0x55cb05e5333e59e535c81183d5a4f7f8b2add5679996c5d426b0bbb6665b564";

    const ANYONE_TEST_ACCOUNT: &str =
        "0x65f1506b7f974a1355aeebc1314579326c84a029cd8257a91f82384a6a0ace";

    const BADGE_REGISTRY_ADDRESS: &str =
        "0x0689c0f3483daffd4e79a61f22f5a093f8adee50926a96161c23b058de70200d";

    const HASH: &str = "0x287b943b1934949486006ad63ac0293038b6c818b858b09f8e0a9da12fc4074";
    const SIGNATURE_R: &str = "0xde4d49b21dd8714eaf5a1b480d8ede84d2230d1763cfe06762d8a117493bcd";
    const SIGNATURE_S: &str = "0x4b61402b98b29a34bd4cba8b5eabae840809914160002385444059f59449a4";
    const BAD_SIGNATURE_R: &str =
        "0x000049b21dd8714eaf5a1b480d8ede84d2230d1763cfe06762d8a117490000";

    #[tokio::test]
    async fn check_signature_is_valid() {
        let client = StarkNetClient::new(
            ADMIN_TEST_ACCOUNT,
            ADMIN_TEST_PRIVATE_KEY,
            BADGE_REGISTRY_ADDRESS,
            StarkNetChain::Testnet,
        );

        let address = FieldElement::from_hex_be(ANYONE_TEST_ACCOUNT).unwrap();
        let hash = FieldElement::from_hex_be(HASH).unwrap();
        let signature_r = FieldElement::from_hex_be(SIGNATURE_R).unwrap();
        let signature_s = FieldElement::from_hex_be(SIGNATURE_S).unwrap();

        let result = client
            .check_signature(
                contracts::badge_registry::SignedData {
                    hash,
                    signature: Signature {
                        r: signature_r,
                        s: signature_s,
                    },
                },
                address,
            )
            .await;

        assert!(result.is_ok(), "{}", result.err().unwrap());
    }

    #[tokio::test]
    async fn check_signature_is_not_valid() {
        let client = StarkNetClient::new(
            ADMIN_TEST_ACCOUNT,
            ADMIN_TEST_PRIVATE_KEY,
            BADGE_REGISTRY_ADDRESS,
            StarkNetChain::Testnet,
        );

        let address = FieldElement::from_hex_be(ANYONE_TEST_ACCOUNT).unwrap();
        let hash = FieldElement::from_hex_be(HASH).unwrap();
        let signature_r = FieldElement::from_hex_be(BAD_SIGNATURE_R).unwrap();
        let signature_s = FieldElement::from_hex_be(SIGNATURE_S).unwrap();

        let result = client
            .check_signature(
                contracts::badge_registry::SignedData {
                    hash,
                    signature: Signature {
                        r: signature_r,
                        s: signature_s,
                    },
                },
                address,
            )
            .await;

        assert!(&result.is_err());
        match result.err().unwrap() {
            crate::contracts::errors::StarknetError::TransactionError(e) => {
                assert!(e
                    .to_string()
                    .contains("is invalid, with respect to the public key"))
            }
        }
    }

    #[tokio::test]
    async fn register_user() {
        let client = StarkNetClient::new(
            ADMIN_TEST_ACCOUNT,
            ADMIN_TEST_PRIVATE_KEY,
            BADGE_REGISTRY_ADDRESS,
            StarkNetChain::Testnet,
        );

        let user_address = FieldElement::from_hex_be(ANYONE_TEST_ACCOUNT).unwrap();

        let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let user_id: u64 = since_the_epoch.as_millis().try_into().unwrap();

        let result = client.register_user(user_address, user_id).await;
        assert!(result.is_ok(), "{}", result.err().unwrap());
    }
}
