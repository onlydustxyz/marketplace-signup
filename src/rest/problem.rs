use rocket::{
    http::Status,
    response::status,
    serde::{json::Json, Serialize},
};
use serde_with::serde_as;

// Error response following the "Problem Details for HTTP APIs" RFC: https://datatracker.ietf.org/doc/html/rfc7807
#[serde_as]
#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Problem {
    title: String,
    detail: String,
    status: u16,
}

pub type ProblemResponse = status::Custom<Json<Problem>>;

impl Problem {
    pub fn new(status_code: u16, title: &str, details: String) -> Problem {
        Problem {
            title: title.to_string(),
            detail: details,
            status: status_code,
        }
    }
}

pub fn new_response(status: Status, title: &str, details: String) -> ProblemResponse {
    status::Custom(status, Json(Problem::new(status.code, title, details)))
}
