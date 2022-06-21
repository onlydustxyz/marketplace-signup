pub mod github;

#[rocket::async_trait]
pub trait IdentityProvider: Send + Sync {
    async fn new_access_token(&self, authorization_code: &str) -> Result<String, reqwest::Error>;

    async fn get_user_id(&self, access_token: &str) -> Result<u64, reqwest::Error>;
}
