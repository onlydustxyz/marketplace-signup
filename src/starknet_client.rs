use std::{
    fmt::{self},
    str::FromStr,
};

use starknet::{
    accounts::single_owner::TransactionError,
    core::{
        types::{BlockId, FieldElement, InvokeFunctionTransactionRequest},
        utils::get_selector_from_name,
    },
    providers::{Provider, SequencerGatewayProvider},
    signers::{LocalWallet, Signer},
};

pub struct StarkNetClient {
    provider: SequencerGatewayProvider,
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

#[derive(Debug)]
pub enum StarknetError {
    TransactionError(
        TransactionError<
            <SequencerGatewayProvider as Provider>::Error,
            <LocalWallet as Signer>::SignError,
        >,
    ),
}

impl fmt::Display for StarknetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StarknetError::TransactionError(e) => e.fmt(f),
        }
    }
}

pub enum StarkNetChain {
    Testnet,
    Mainnet,
}

impl FromStr for StarkNetChain {
    type Err = ();

    fn from_str(input: &str) -> Result<StarkNetChain, Self::Err> {
        match input {
            "TESTNET" => Ok(StarkNetChain::Testnet),
            "MAINNET" => Ok(StarkNetChain::Mainnet),
            _ => Err(()),
        }
    }
}

impl StarkNetClient {
    pub fn new(chain: StarkNetChain) -> Self {
        let provider = match chain {
            StarkNetChain::Testnet => SequencerGatewayProvider::starknet_alpha_goerli(),
            StarkNetChain::Mainnet => SequencerGatewayProvider::starknet_alpha_mainnet(),
        };

        StarkNetClient { provider }
    }

    pub async fn check_signature(
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
}

#[cfg(test)]
mod tests {
    use super::Signature;
    use super::StarkNetChain;
    use super::StarkNetClient;
    use rocket::tokio;
    use starknet::core::types::FieldElement;

    const ANYONE_TEST_ACCOUNT: &str =
        "0x65f1506b7f974a1355aeebc1314579326c84a029cd8257a91f82384a6a0ace";

    const HASH: &str = "0x287b943b1934949486006ad63ac0293038b6c818b858b09f8e0a9da12fc4074";
    const SIGNATURE_R: &str = "0xde4d49b21dd8714eaf5a1b480d8ede84d2230d1763cfe06762d8a117493bcd";
    const SIGNATURE_S: &str = "0x4b61402b98b29a34bd4cba8b5eabae840809914160002385444059f59449a4";
    const BAD_SIGNATURE_R: &str =
        "0x000049b21dd8714eaf5a1b480d8ede84d2230d1763cfe06762d8a117490000";

    #[tokio::test]
    async fn check_signature_is_valid() {
        let client = StarkNetClient::new(StarkNetChain::Testnet);

        let address = FieldElement::from_hex_be(ANYONE_TEST_ACCOUNT).unwrap();
        let hash = FieldElement::from_hex_be(HASH).unwrap();
        let signature_r = FieldElement::from_hex_be(SIGNATURE_R).unwrap();
        let signature_s = FieldElement::from_hex_be(SIGNATURE_S).unwrap();

        let result = client
            .check_signature(
                super::SignedData {
                    hash,
                    signature: Signature {
                        r: signature_r,
                        s: signature_s,
                    },
                },
                address,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn check_signature_is_not_valid() {
        let client = StarkNetClient::new(StarkNetChain::Testnet);

        let address = FieldElement::from_hex_be(ANYONE_TEST_ACCOUNT).unwrap();
        let hash = FieldElement::from_hex_be(HASH).unwrap();
        let signature_r = FieldElement::from_hex_be(BAD_SIGNATURE_R).unwrap();
        let signature_s = FieldElement::from_hex_be(SIGNATURE_S).unwrap();

        let result = client
            .check_signature(
                super::SignedData {
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
            crate::starknet_client::StarknetError::TransactionError(e) => {
                assert!(e
                    .to_string()
                    .contains("is invalid, with respect to the public key"))
            }
        }
    }
}
