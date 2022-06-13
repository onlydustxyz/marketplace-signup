use starknet::core::{
    crypto::compute_hash_on_elements, types::FieldElement, utils::cairo_short_string_to_felt,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HashCheckError {
    #[error("wrong hash (expected {expected_hash:?}, was {actual_hash:?})")]
    WrongHash {
        expected_hash: FieldElement,
        actual_hash: FieldElement,
    },
}

#[derive(Error, Debug)]
pub enum StringHashCheckError {
    #[error(transparent)]
    CairoShortStringToFeltError(#[from] starknet::core::utils::CairoShortStringToFeltError),
    #[error(transparent)]
    WrongHash(#[from] HashCheckError),
}

pub fn check_pedersen_hash_str(
    data_str: &str,
    hex_expected_hash: &str,
) -> Result<(), StringHashCheckError> {
    let data = cairo_short_string_to_felt(data_str)?;
    let expected_hash =
        FieldElement::from_hex_be(hex_expected_hash).expect("Invalid expected hash");
    let result = check_pedersen_hash(&[data], expected_hash)?;
    Ok(result)
}

pub fn check_pedersen_hash(
    data: &[FieldElement],
    expected_hash: FieldElement,
) -> Result<(), HashCheckError> {
    let actual_hash = compute_hash_on_elements(data);

    if actual_hash != expected_hash {
        return Err(HashCheckError::WrongHash {
            expected_hash,
            actual_hash,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check_pedersen_hash_str;
    use rocket::tokio;

    const DATA: &str = "6ec1a38b374a060cb11c";
    const DATA_HASH: &str = "0x728a87506720701bf7f5e1f51b395229dab9904dbf09848cc36752732517803";

    #[tokio::test]
    async fn test_check_pedersen_hash() {
        let result = check_pedersen_hash_str(DATA, DATA_HASH);
        assert!(result.is_ok(), "{}", result.err().unwrap());
    }
}
