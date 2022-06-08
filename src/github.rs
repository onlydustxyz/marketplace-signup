use rocket::serde::{Deserialize, Serialize};

pub struct GitHubClient {
    http_client: reqwest::Client,

    access_token_url: String,
    user_api_url: String,

    github_id: String,
    github_secret: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct AccessTokenRequestBody<'r> {
    client_id: &'r str,
    client_secret: &'r str,
    code: &'r str,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct AccessTokenResponseBody {
    access_token: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct UserResponseBody {
    id: u64,
}

impl GitHubClient {
    pub fn new(
        github_id: String,
        github_secret: String,
        access_token_url: String,
        user_api_url: String,
    ) -> Self {
        GitHubClient {
            http_client: reqwest::Client::new(),
            access_token_url,
            user_api_url,
            github_id,
            github_secret,
        }
    }

    pub async fn new_access_token(
        &self,
        authorization_code: &str,
    ) -> Result<String, reqwest::Error> {
        let request_body = AccessTokenRequestBody {
            client_id: &self.github_id,
            client_secret: &self.github_secret,
            code: authorization_code,
        };

        let response = self
            .http_client
            .post(&self.access_token_url)
            .json(&request_body)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await?
            .error_for_status()?;

        let response = response.json::<AccessTokenResponseBody>().await?;
        Ok(response.access_token)
    }

    pub async fn get_user_id(&self, access_token: &str) -> Result<u64, reqwest::Error> {
        let response = self
            .http_client
            .get(&self.user_api_url)
            .header(reqwest::header::ACCEPT, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("token {}", access_token),
            )
            .send()
            .await?
            .error_for_status()?;

        let response = response.json::<UserResponseBody>().await?;
        Ok(response.id)
    }
}

#[cfg(test)]
mod tests {
    use super::GitHubClient;
    use claim::*;
    use httpmock::prelude::*;
    use rocket::{serde::json::serde_json, tokio};
    use serde_json::json;

    #[tokio::test]
    async fn new_access_token() {
        // Start a server running on a local ephemeral port.
        let server = MockServer::start();

        let github_client = GitHubClient::new(
            "foo-github-id".into(),
            "foo-github-secret".into(),
            server.url("/login/oauth/access_token"),
            "".into(),
        );

        let github_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/login/oauth/access_token")
                .header("Accept", "application/json")
                .json_body(json!({
                    "client_id": "foo-github-id",
                    "client_secret": "foo-github-secret",
                    "code": "foo-code",
                }));
            then.status(200).json_body(json!({
                "access_token":"gho_16C7e42F292c6912E7710c838347Ae178B4a",
                "scope":"repo,gist",
                "token_type":"bearer",
            }));
        });

        let result = github_client.new_access_token("foo-code").await;

        github_mock.assert();
        assert!(result.is_ok());
        assert_eq!(
            "gho_16C7e42F292c6912E7710c838347Ae178B4a".to_string(),
            result.unwrap()
        );
    }

    #[tokio::test]
    async fn get_user_id() {
        // Start a server running on a local ephemeral port.
        let server = MockServer::start();

        let github_client = GitHubClient::new(
            "foo-github-id".into(),
            "foo-github-secret".into(),
            "".into(),
            server.url("/user"),
        );

        let github_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/user")
                .header("Accept", "application/json")
                .header("Authorization", "token foo-access-token");
            then.status(200).json_body(json!({
                "login": "octocat",
                "id": 42,
                "node_id": "MDQ6VXNlcjE=",
            }));
        });

        let result = github_client.get_user_id("foo-access-token").await;

        github_mock.assert();
        assert_ok_eq!(result, 42);
    }
}
