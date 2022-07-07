use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use starknet::{
    accounts::SingleOwnerAccount,
    core::{
        chain_id::{MAINNET, TESTNET},
        types::FieldElement,
    },
    providers::SequencerGatewayProvider,
    signers::{LocalWallet, SigningKey},
};

pub struct StarkNetClient {
    pub provider: SequencerGatewayProvider,
    pub account: SingleOwnerAccount<SequencerGatewayProvider, LocalWallet>,
    pub badge_registry_address: FieldElement,
}

impl StarkNetClient {
    pub fn new(
        hex_account_address: &str,
        hex_private_key: &str,
        hex_badge_registry_address: &str,
        chain: StarkNetChain,
    ) -> Self {
        let provider = match chain {
            StarkNetChain::Testnet => SequencerGatewayProvider::starknet_alpha_goerli(),
            StarkNetChain::Mainnet => SequencerGatewayProvider::starknet_alpha_mainnet(),
        };
        let account_provider = match chain {
            StarkNetChain::Testnet => SequencerGatewayProvider::starknet_alpha_goerli(),
            StarkNetChain::Mainnet => SequencerGatewayProvider::starknet_alpha_mainnet(),
        };
        let chain_id = match chain {
            StarkNetChain::Testnet => TESTNET,
            StarkNetChain::Mainnet => MAINNET,
        };
        let signer = LocalWallet::from(SigningKey::from_secret_scalar(
            FieldElement::from_hex_be(hex_private_key).expect("Invalid private key"),
        ));
        let account_address =
            FieldElement::from_hex_be(hex_account_address).expect("Invalid account address");
        let badge_registry_address = FieldElement::from_hex_be(hex_badge_registry_address)
            .expect("Invalid address for badge_registry");

        StarkNetClient {
            provider,
            account: SingleOwnerAccount::new(account_provider, signer, account_address, chain_id),
            badge_registry_address,
        }
    }

    pub fn get_timestamp_based_nonce(&self) -> FieldElement {
        let start = SystemTime::now();
        let timestamp = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        FieldElement::from_dec_str(&timestamp.as_micros().to_string()).unwrap()
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
