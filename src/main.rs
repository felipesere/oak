use rocket::{Build, Rocket};

mod pokeapi;

#[rocket::get("/pokemon/<name>")]
async fn find_pokemon(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[rocket::launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", rocket::routes![find_pokemon])
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::http::Status;

    #[test]
    fn sketch_of_how_to_use_rocket_testing_facilities() {
        use rocket::local::blocking::Client;

        let client = Client::tracked(rocket()).unwrap();
        let response = client.get("/pokemon/mewtwo").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("Hello, mewtwo!".into()));
    }
}
