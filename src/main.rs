use pokeapi::{PokeApiSettings, PokeClient};
use rocket::http::Status;
use rocket::{serde::json::Json, Build, Rocket, State};
use serde::Serialize;
use translation::{Language, TranslationClient, TranslationSettings};

mod pokeapi;
mod translation;

#[derive(Serialize)]
struct ApiError {
    message: String,
}

type ApiResult<T> = Result<Json<T>, (Status, Json<ApiError>)>;

fn ok<T>(value: T) -> ApiResult<T> {
    Result::Ok(Json(value))
}

fn not_found<T>(message: String) -> ApiResult<T> {
    Result::Err((Status::NotFound, Json(ApiError { message })))
}

#[rocket::get("/pokemon/<name>")]
async fn find_pokemon(poke_api: &State<PokeClient>, name: &str) -> ApiResult<Pokemon> {
    match poke_api.find(name).await {
        Ok(pokemon) => ok(pokemon),
        Err(_) => not_found(format!("Unable to find '{}'", name)),
    }
}

#[rocket::get("/pokemon/translated/<name>")]
async fn find_translated_pokemon(
    poke_api: &State<PokeClient>,
    translation_api: &State<TranslationClient>,
    name: &str,
) -> ApiResult<Pokemon> {
    match poke_api.find(name).await {
        Ok(mut pokemon) => {
            let translated = translation_api
                .translate(&pokemon.description, Language::Yoda)
                .await
                .expect("Error when translating");
            pokemon.description = translated;
            ok(pokemon)
        }
        Err(_) => not_found(format!("Unable to find '{}'", name)),
    }
}

struct Settings {
    poke_api: PokeApiSettings,
    translation_api: TranslationSettings,
}

impl Settings {
    fn poke_api_client(&self) -> PokeClient {
        self.poke_api.clone().into()
    }

    fn translation_api_client(&self) -> TranslationClient {
        self.translation_api.clone().into()
    }
}

fn rocket(settings: Settings) -> Rocket<Build> {
    let poke_api_client = settings.poke_api_client();
    let translation_client = settings.translation_api_client();

    rocket::build()
        .manage(poke_api_client)
        .manage(translation_client)
        .mount("/", rocket::routes![find_pokemon, find_translated_pokemon])
}
#[rocket::main]
async fn main() {
    use std::time::Duration;

    let settings = Settings {
        poke_api: PokeApiSettings {
            base_url: "https://pokeapi.co".to_string(),
            timeout: Duration::from_secs(10),
        },
        translation_api: TranslationSettings {
            base_url: "https://api.funtranslations.com".into(),
            timeout: Duration::from_secs(10),
        },
    };

    let _ = rocket(settings).launch().await;
}

// TODO: Consider if I want a similar layer between this external facing "API Pokemon"
// and the internal "Pokemon"
#[derive(Debug, Serialize)]
pub struct Pokemon {
    pub name: String,
    pub description: String,
    pub habitat: String,
    #[serde(rename = "isLegendary")]
    pub is_legendary: bool,
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

    #[test]
    fn serializes_pokemon_responses_to_the_adequate_json() {
        let mewtwo = Pokemon {
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

    const RAW_MEWTWO: &'static str = include_str!("../examples/mewtwo.json");
    const RAW_DIGLETT: &'static str = include_str!("../examples/diglett.json");
    const DIGLETT_AS_SHAKESPEAR: &'static str =
        include_str!("../examples/translation/diglett_shakespeare.json");

    // TODO: Consider wrapping these to MockServer into custom types?
    async fn setup() -> (Settings, MockServer, MockServer) {
        let poke_server = MockServer::start().await;

        let translation_server = MockServer::start().await;

        let settings = Settings {
            poke_api: PokeApiSettings {
                base_url: format!("http://{}", poke_server.address().to_string()),
                timeout: std::time::Duration::from_secs(3),
            },
            translation_api: TranslationSettings {
                base_url: format!("http://{}", translation_server.address().to_string()),
                timeout: std::time::Duration::from_secs(3),
            },
        };

        (settings, poke_server, translation_server)
    }

    #[tokio::test]
    async fn requesting_mewtwo_makes_a_call_to_the_pokemon_api() {
        let (settings, mock_poke_api, _) = setup().await;

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
                    "description": "It was created by a scientist after years of horrific gene splicing and DNA engineering experiments.",
                    "habitat":"rare",
                    "isLegendary":true
                }
                "#
            )
        );
    }

    #[tokio::test]
    async fn api_errors_contain_a_indicateive_message() {
        let (settings, mock_poke_api, _) = setup().await;

        // There are no Pokemons :(
        Mock::given(any())
            .respond_with(ResponseTemplate::new(404))
            .expect(1)
            .mount(&mock_poke_api)
            .await;

        let client = Client::tracked(rocket(settings)).await.unwrap();

        let response = client.get("/pokemon/mewtwo").dispatch().await;
        assert_eq!(response.status(), Status::NotFound);
        let error = response
            .into_string()
            .await
            .expect("Unexpected empty response");

        assert_json_eq!(
            json(&error),
            json(
                r#"
                {
                    "message": "Unable to find 'mewtwo'"
                }
                "#
            )
        );
    }

    #[tokio::test]
    async fn when_asking_for_a_translation_the_description_of_a_cave_pokemon_is_in_yoda_speak() {
        let (settings, mock_poke_api, mock_translation_api) = setup().await;

        // diglett is the best cave Pokemon
        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/diglett"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(RAW_DIGLETT, "application/json"))
            .expect(1)
            .mount(&mock_poke_api)
            .await;

        // TODO: Check the body of the post at least has the '.text' key!
        Mock::given(method("POST"))
            .and(path("/translate/yoda"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(DIGLETT_AS_SHAKESPEAR, "application/json"),
            )
            .expect(1)
            .mount(&mock_translation_api)
            .await;

        let client = Client::tracked(rocket(settings)).await.unwrap();

        let response = client.get("/pokemon/translated/diglett").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        let diglett_json = response
            .into_string()
            .await
            .expect("Unexpected empty response");

        assert_json_eq!(
            json(&diglett_json),
            json(
                r#"
                {
                    "name": "diglett",
                    "description": "On plant roots,  lives about one yard underground where it feeds.Above ground,  it sometimes appears.",
                    "habitat":"cave",
                    "isLegendary":false
                }
                "#
            )
        );
    }
}
