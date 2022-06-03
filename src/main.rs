#[macro_use]
extern crate rocket;

#[post("/")]
fn create_badge() -> &'static str {
    "Hello, badge!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/badges", routes![create_badge])
}
