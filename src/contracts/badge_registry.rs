use anyhow::Result;
use starknet::{
	accounts::{single_owner::TransactionError, Account, AccountCall, Call},
	core::{
		types::{AddTransactionResult, BlockId, FieldElement, InvokeFunctionTransactionRequest},
		utils::get_selector_from_name,
	},
	providers::Provider,
};

use super::{client::StarkNetClient, errors::StarknetError};

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
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
	) -> Result<AddTransactionResult>;
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
	) -> Result<AddTransactionResult> {
		let nonce = self.get_2d_nonce(FieldElement::from(github_user_id)).await?;

		self.account
			.execute(&[Call {
				to: self.badge_registry_address,
				selector: get_selector_from_name("register_github_identifier").unwrap(),
				calldata: vec![user_account_address, FieldElement::from(github_user_id)],
			}])
			.nonce(nonce)
			.send()
			.await
			.map_err(anyhow::Error::msg)
	}
}

#[cfg(test)]
mod tests {
	use crate::contracts::{
		self,
		badge_registry::{BadgeRegistryClient, Signature},
		client::{StarkNetChain, StarkNetClient},
	};

	use dotenv::dotenv;
	use rand::prelude::*;
	use rocket::tokio;
	use starknet::core::types::FieldElement;

	const ANYONE_TEST_ACCOUNT: &str =
		"0x65f1506b7f974a1355aeebc1314579326c84a029cd8257a91f82384a6a0ace";

	const REGISTRY_ADDRESS: &str =
		"0x04e16efc9bc2d8d40ecb73d3d69e3e2d6f0fc3e2e6e9b7601310fdfa7dd6c7cf";

	const HASH: &str = "0x287b943b1934949486006ad63ac0293038b6c818b858b09f8e0a9da12fc4074";
	const SIGNATURE_R: &str = "0xde4d49b21dd8714eaf5a1b480d8ede84d2230d1763cfe06762d8a117493bcd";
	const SIGNATURE_S: &str = "0x4b61402b98b29a34bd4cba8b5eabae840809914160002385444059f59449a4";
	const BAD_SIGNATURE_R: &str =
		"0x000049b21dd8714eaf5a1b480d8ede84d2230d1763cfe06762d8a117490000";

	fn new_test_client() -> StarkNetClient {
		dotenv().ok();
		let admin_account = std::env::var("STARKNET_ACCOUNT").unwrap();
		let admin_private_key = std::env::var("STARKNET_PRIVATE_KEY").unwrap();

		StarkNetClient::new(
			admin_account.as_str(),
			admin_private_key.as_str(),
			REGISTRY_ADDRESS,
			StarkNetChain::Testnet,
		)
	}

	#[tokio::test]
	async fn check_signature_is_valid() {
		let client = new_test_client();

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
		let client = new_test_client();

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
				assert!(e.to_string().contains("is invalid, with respect to the public key"))
			},
		}
	}

	#[tokio::test]
	async fn register_user() {
		let client = new_test_client();

		// use very high ids to avoid any conflict with real github ids
		let user_id: u64 = std::u64::MAX - rand::thread_rng().gen_range(1_000..1_000_000_000);
		let user_address = FieldElement::from(user_id - 42);

		let transaction_result = client.register_user(user_address, user_id).await;
		assert!(
			transaction_result.is_ok(),
			"{}",
			transaction_result.err().unwrap()
		);

		let acceptance_result =
			client.wait_for_transaction_acceptance(transaction_result.unwrap()).await;
		assert!(
			acceptance_result.is_ok(),
			"{}",
			acceptance_result.err().unwrap()
		);
	}

	#[tokio::test]
	async fn register_multiple_user_concurrently() {
		let client = new_test_client();

		// use very high ids to avoid any conflict with real github ids
		let mut user_id: u64 = std::u64::MAX - rand::thread_rng().gen_range(1_000..1_000_000_000);

		let mut transactions = Vec::new();

		for _ in 0..5 {
			let user_address = FieldElement::from(user_id - 42);
			let transaction_result = client.register_user(user_address, user_id).await;
			assert!(
				transaction_result.is_ok(),
				"{}",
				transaction_result.err().unwrap()
			);
			transactions.push(transaction_result.unwrap());
			user_id += 1;
		}

		for transaction in transactions {
			let acceptance_result = client.wait_for_transaction_acceptance(transaction).await;
			assert!(
				acceptance_result.is_ok(),
				"{}",
				acceptance_result.err().unwrap()
			);
		}
	}
}
