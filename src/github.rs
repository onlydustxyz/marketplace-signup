use rocket::serde::{Deserialize, Serialize};

pub struct GitHubClient {
    http_client: reqwest::Client,

    access_token_url: String,
    user_url: String,

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
    pub fn new() -> GitHubClient {
        GitHubClient {
            http_client: reqwest::Client::new(),
            access_token_url: "https://github.com/login/oauth/access_token".into(),
            user_url: "https://api.github.com/user".into(),
            github_id: "github_id".into(),
            github_secret: "github_secret".into(),
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
            .header(reqwest::header::ACCEPT, "application/json")
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        let response = response.json::<AccessTokenResponseBody>().await?;
        Ok(response.access_token)
    }

    pub async fn get_user_id(&self, access_token: &str) -> Result<u64, reqwest::Error> {
        let response = self
            .http_client
            .get(&self.user_url)
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
    use claim::assert_ok_eq;
    use httptest::{matchers::*, responders::*, Expectation, Server};
    use rocket::tokio;

    #[tokio::test]
    async fn new_access_token() {
        // Start a server running on a local ephemeral port.
        let server = Server::run();

        let mut github_client = GitHubClient::new();
        github_client.access_token_url = server.url("/login/oauth/access_token").to_string();

        server.expect(
            Expectation::matching(request::method_path("POST", "/login/oauth/access_token"))
                .respond_with(status_code(200).body(
                    r#"{
                        "access_token":"gho_16C7e42F292c6912E7710c838347Ae178B4a",
                        "scope":"repo,gist",
                        "token_type":"bearer"
                  }"#,
                )),
        );

        let authorization_code = "foo-authorization_code";
        let result = github_client.new_access_token(authorization_code).await;
        assert!(result.is_ok());
        assert_eq!(
            "gho_16C7e42F292c6912E7710c838347Ae178B4a".to_string(),
            result.unwrap()
        );
    }

    #[tokio::test]
    async fn get_user_id() {
        // Start a server running on a local ephemeral port.
        let server = Server::run();

        let mut github_client = GitHubClient::new();
        github_client.user_url = server.url("/user").to_string();

        server.expect(
            Expectation::matching(request::method_path("GET", "/user")).respond_with(
                status_code(200).body(
                    r#"{
                        "login": "octocat",
                        "id": 42,
                        "node_id": "MDQ6VXNlcjE="
                  }"#,
                ),
            ),
        );

        let access_token = "foo-access_token";
        let result = github_client.get_user_id(access_token).await;
        assert_ok_eq!(result, 42);
    }
}
