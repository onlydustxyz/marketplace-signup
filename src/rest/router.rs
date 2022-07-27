use rocket::{Build, Rocket};
use rocket_okapi::{
    openapi_get_routes,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};

use crate::{
    application::registerer::Registerer,
    infrastructure::{github_client::GitHubClient, starknet_client::StarkNetClient},
};

pub fn new(registerer: Box<dyn Registerer<GitHubClient, StarkNetClient>>) -> Rocket<Build> {
    rocket::build()
        .manage(registerer)
        .attach(super::cors::Cors)
        .mount(
            "/",
            routes![
                super::cors::options_preflight_handler,
                super::health::health_check
            ],
        )
        .mount(
            "/",
            openapi_get_routes![super::registrations::register_github_user],
        )
        .mount("/swagger", make_swagger_ui(&get_docs()))
}

pub(crate) fn get_docs() -> SwaggerUIConfig {
    SwaggerUIConfig {
        url: "/openapi.json".to_string(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;
    use rocket::{http::Status, local::blocking::Client};

    use crate::{
        application::registerer::Registerer,
        domain::{errors::RegistrationError, services::onchain_registry::OnChainRegistry},
        infrastructure::{github_client::GitHubClient, starknet_client::StarkNetClient},
        rest,
    };

    mock! {
        MyRegisterer {}
        #[async_trait]
        impl Registerer<GitHubClient, StarkNetClient> for MyRegisterer {
            async fn register_contributor(
                &self,
                authorization_code: String,
                account_address: <StarkNetClient as OnChainRegistry>::AccountAddress,
                signed_data: <StarkNetClient as OnChainRegistry>::SignedData,
            ) -> Result<<StarkNetClient as OnChainRegistry>::TransactionHash, RegistrationError>;
        }
    }

    #[test]
    fn test_options() {
        let registerer_mock = MockMyRegisterer::new();

        let router = rest::router::new(
            Box::new(registerer_mock) as Box<dyn Registerer<GitHubClient, StarkNetClient>>
        );

        let client = Client::tracked(router).expect("valid rocket instance");
        let response = client.options(uri!("/registrations/github")).dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.headers().get_one("Access-Control-Allow-Origin"),
            Some("*")
        );
        assert_eq!(
            response.headers().get_one("Access-Control-Allow-Methods"),
            Some("POST, PUT, GET, PATCH, OPTIONS")
        );
        assert_eq!(
            response.headers().get_one("Access-Control-Allow-Headers"),
            Some("*")
        );
        assert_eq!(
            response
                .headers()
                .get_one("Access-Control-Allow-Credentials"),
            Some("true")
        );
    }
}
