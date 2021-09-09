use pokeapi::{PokeApiSettings, PokeClient};
use rocket::{serde::json::Json, Build, Rocket, State};
use serde::Serialize;

mod pokeapi;

#[rocket::get("/pokemon/<name>")]
async fn find_pokemon(_poke_api: &State<PokeClient>, name: &str) -> Json<PokemonResponse> {
    Json(PokemonResponse {
        name: name.into(),
        description: "It was created by scientists after years...".into(),
        habitat: "rare".into(),
        is_legendary: true,
    })
}

#[rocket::launch]
fn rocket() -> Rocket<Build> {
    // TODO: This is a placeholder until configuration is sorted
    let poke_api_client: PokeClient = PokeApiSettings {
        base_url: "https://pokeapi.co".to_string(),
        timeout: std::time::Duration::from_secs(10),
    }
    .into();

    rocket::build()
        .manage(poke_api_client)
        .mount("/", rocket::routes![find_pokemon])
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
        let mewtwo_json = response.into_string().expect("Unexpected empty response");
        assert_json_eq!(
            json(&mewtwo_json),
            json(
                r#"
                {
                    "name": "mewtwo",
                    "description": "It was created by scientists after years...",
                    "habitat":"rare",
                    "isLegendary":true
                }
                "#
            )
        );
    }

    #[test]
    fn serializes_pokemon_responses_to_the_adequate_json() {
        let mewtwo = PokemonResponse {
            name: "mewtwo".into(),
            description: "It was created by scientists after years...".into(),
            habitat: "rare".into(),
            is_legendary: true,
        };

        let actual_json =
            serde_json::to_string(&mewtwo).expect("unable to serialize MewTwo to JSON");

        assert_json_eq!(
            json(&actual_json),
            json(
                r#"
                {
                    "name": "mewtwo",
                    "description": "It was created by scientists after years...",
                    "habitat":"rare",
                    "isLegendary":true
                }
                "#
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
