use rocket::{Build, Rocket};

use crate::{contracts::badge_registry::BadgeRegistryClient, identity_providers::IdentityProvider};

pub fn new(
    github_identity_provider: Box<dyn IdentityProvider>,
    badge_registry_client: Box<dyn BadgeRegistryClient>,
) -> Rocket<Build> {
    rocket::build()
        .manage(github_identity_provider)
        .manage(badge_registry_client)
        .mount("/", routes![super::health::health_check])
        .mount(
            "/registrations",
            routes![super::registrations::register_github_user],
        )
}
