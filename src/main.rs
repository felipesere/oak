use rocket::{Build, Rocket};
use serde::Serialize;

mod pokeapi;

#[rocket::get("/pokemon/<name>")]
async fn find_pokemon(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[rocket::launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", rocket::routes![find_pokemon])
}

#[derive(Serialize)]
struct PokemonResponse {
    name: String,
    description: String,
    habitat: String,
    #[serde(rename = "isLegendary")]
    is_legendary: bool,
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_json_diff::assert_json_eq;
    use rocket::http::Status;

    #[test]
    fn sketch_of_how_to_use_rocket_testing_facilities() {
        use rocket::local::blocking::Client;

        let client = Client::tracked(rocket()).unwrap();
        let response = client.get("/pokemon/mewtwo").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("Hello, mewtwo!".into()));
    }

    #[test]
    fn serializes_pokemon_responses_to_the_adequate_json() {
        let mewtwo = PokemonResponse {
            name: "mewtwo".into(),
            description: "It was created by scientists after years...".into(),
            habitat: "rare".into(),
            is_legendary: true,
        };

        let mewtwo_json =
            serde_json::to_string(&mewtwo).expect("unable to serialize MewTwo to JSON");

        assert_json_eq!(
            json(&mewtwo_json),
            json(
                r#"{"name":"mewtwo","description":"It was created by scientists after years...","habitat":"rare","isLegendary":true}"#
            )
        );
    }

    fn json(input: &str) -> serde_json::Value {
        match serde_json::from_str::<serde_json::Value>(input) {
            Ok(value) => value,
            Err(err) => {
                panic!("Did not get valid JSON: {}. Context\n{}", err, input)
            }
        }
    }
}
