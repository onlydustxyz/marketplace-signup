use std::fmt;

use rocket::serde::{self, Deserialize, Serialize, Serializer};
use starknet::core::types::FieldElement;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct HexFieldElement {
    pub felt: FieldElement,
}

impl fmt::Display for HexFieldElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.felt.fmt(f)
    }
}

impl<'de> Deserialize<'de> for HexFieldElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let felt = FieldElement::from_hex_be(&value);
        match felt {
            Ok(felt) => Ok(HexFieldElement { felt }),
            Err(err) => Err(serde::de::Error::custom(format!(
                "invalid hexadecimal felt string: {}",
                err
            ))),
        }
    }
}

impl<'de> Serialize for HexFieldElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:#x}", self.felt))
    }
}

impl From<HexFieldElement> for FieldElement {
    fn from(data: HexFieldElement) -> Self {
        data.felt
    }
}

impl From<FieldElement> for HexFieldElement {
    fn from(data: FieldElement) -> Self {
        Self { felt: data }
    }
}

#[cfg(test)]
mod tests {
    use super::HexFieldElement;
    use rocket::{
        serde::{json::serde_json, Deserialize, Serialize},
        tokio,
    };
    use starknet::core::types::FieldElement;

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(crate = "rocket::serde")]
    pub struct Container {
        pub field: HexFieldElement,
    }

    const HEX_FELT: &str = "0x65f1506b7f974a1355aeebc1314579326c84a029cd8257a91f82384a6a0ace";

    #[tokio::test]
    async fn test_serialize_deserialize() {
        let container = Container {
            field: HexFieldElement {
                felt: FieldElement::from_hex_be(HEX_FELT).unwrap(),
            },
        };

        // Convert the Container to a JSON string.
        let serialized = serde_json::to_string(&container).unwrap();
        assert_eq!(
            "{\"field\":\"0x65f1506b7f974a1355aeebc1314579326c84a029cd8257a91f82384a6a0ace\"}"
                .to_string(),
            serialized
        );

        // Convert the JSON string back to a Container.
        let deserialized: Container = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            "180117042528829806066732664429515383901753484451094593570285525801139571406",
            deserialized.field.felt.to_string()
        );
    }

    #[tokio::test]
    async fn test_from() {
        let felt = FieldElement::from_hex_be(HEX_FELT).unwrap();

        let hex_felt = HexFieldElement::from(felt);
        assert_eq!(felt, hex_felt.felt);

        let from_hex_felt = FieldElement::from(hex_felt);
        assert_eq!(felt, from_hex_felt);
    }
}
