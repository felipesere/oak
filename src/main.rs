use pokeapi::{PokeApiSettings, PokeClient};
use rocket::{serde::json::Json, Build, Rocket, State};
use serde::Serialize;

mod pokeapi;

#[rocket::get("/pokemon/<name>")]
async fn find_pokemon(poke_api: &State<PokeClient>, name: &str) -> Json<PokemonResponse> {
    // TODO: Use this when we have the Pokemon transformation settled
    let _ = poke_api.find(name).await;

    Json(PokemonResponse {
        name: name.into(),
        description: "It was created by scientists after years...".into(),
        habitat: "rare".into(),
        is_legendary: true,
    })
}

struct Settings {
    poke_api: PokeApiSettings,
}

impl Settings {
    fn poke_api_client(&self) -> PokeClient {
        self.poke_api.clone().into()
    }
}

fn rocket(settings: Settings) -> Rocket<Build> {
    let poke_api_client = settings.poke_api_client();

    rocket::build()
        .manage(poke_api_client)
        .mount("/", rocket::routes![find_pokemon])
}
#[rocket::main]
async fn main() {
    let settings = Settings {
        poke_api: PokeApiSettings {
            base_url: "https://pokeapi.co".to_string(),
            timeout: std::time::Duration::from_secs(10),
        },
    };

    let _ = rocket(settings).launch().await;
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
    use rocket::local::asynchronous::Client;
    use wiremock::{
        matchers::{any, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    const RAW_MEWTWO: &'static str = include_str!("../examples/mewtwo.json");

    async fn setup() -> (Settings, MockServer) {
        let poke_server = MockServer::start().await;

        let settings = Settings {
            poke_api: PokeApiSettings {
                base_url: format!("http://{}", poke_server.address().to_string()),
                timeout: std::time::Duration::from_secs(3),
            },
        };

        (settings, poke_server)
    }

    #[tokio::test]
    async fn requesting_mewtwo_makes_a_call_to_the_pokemon_api() {
        let (settings, mock_poke_api) = setup().await;

        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/mewtwo"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(RAW_MEWTWO, "application/json"))
            .expect(1)
            .mount(&mock_poke_api)
            .await;

        let client = Client::tracked(rocket(settings)).await.unwrap();
        let response = client.get("/pokemon/mewtwo").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        let mewtwo_json = response
            .into_string()
            .await
            .expect("Unexpected empty response");
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
