use crate::domain::{
    errors::AuthenticationError,
    errors::IdentificationError,
    value_objects::{AccessToken, Identity},
};

#[async_trait]
pub trait IdentityProvider: Send + Sync {
    async fn new_access_token(
        &self,
        authorization_code: &str,
    ) -> Result<AccessToken, AuthenticationError>;

    async fn get_user_id(
        &self,
        access_token: &AccessToken,
    ) -> Result<Identity, IdentificationError>;
}
