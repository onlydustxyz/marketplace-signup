use rocket::{Build, Rocket};

use crate::{contracts::badge_registry::BadgeRegistryClient, identity_providers::IdentityProvider};

pub fn new(
	github_identity_provider: Box<dyn IdentityProvider>,
	badge_registry_client: Box<dyn BadgeRegistryClient>,
) -> Rocket<Build> {
	rocket::build()
		.manage(github_identity_provider)
		.manage(badge_registry_client)
		.attach(super::cors::Cors)
		.mount(
			"/",
			routes![
				super::cors::options_preflight_handler,
				super::health::health_check
			],
		)
		.mount(
			"/registrations",
			routes![super::registrations::register_github_user],
		)
}

#[cfg(test)]
mod tests {
	use rocket::{http::Status, local::blocking::Client};

	use crate::{
		contracts::badge_registry::{BadgeRegistryClient, MockBadgeRegistryClient},
		identity_providers::{IdentityProvider, MockIdentityProvider},
		rest,
	};

	#[test]
	fn test_options() {
		let github_mock = MockIdentityProvider::new();
		let badge_registry_mock = MockBadgeRegistryClient::new();

		let router = rest::router::new(
			Box::new(github_mock) as Box<dyn IdentityProvider>,
			Box::new(badge_registry_mock) as Box<dyn BadgeRegistryClient>,
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
			response.headers().get_one("Access-Control-Allow-Credentials"),
			Some("true")
		);
	}
}
