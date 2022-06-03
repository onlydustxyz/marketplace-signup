#[macro_use]
extern crate rocket;
use rocket::serde::{json::Json, Deserialize};
use starknet::core::types::FieldElement;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/registrations", routes![register_github_user])
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct GithubUserRegistration<'r> {
    authorization_code: &'r str,
    signed_authorization_code: &'r str,
    account_address: FieldElement,
}

#[post("/github", format = "json", data = "<registration>")]
fn register_github_user(registration: Json<GithubUserRegistration<'_>>) -> &'static str {
    // TODO: call POST https://github.com/login/oauth/access_token with id+secret+authorization_code
    // to get an access_token (and BTW check that the authorization_code is valid).

    // TODO: call GET https://api.github.com/user with the access_token in the Authorization header to
    // get information about the logged user, in particular, its ID.

    // TODO: call `is_valid_signature` @view function on the Account smart contract (at account_address) to check that
    // the signed_authorization_code is valid and belongs to the given account.

    // TODO: call `register` @external function of the Registry smart contract, passing the github user ID and the account_address
    // as arguments.
    // This call must be done from our own account which has the right to call the `register` function of Registry smart contract.

    "Hello, badge!"
}
