use crate::pokeapi::PokeClient;
use crate::translation::{Language, TranslationClient};
use crate::Settings;

use rocket::http::Status;
use rocket::{serde::json::Json, Build, Rocket, State};
use serde::Serialize;

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
        // TODO: Surface more accurate errors
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
            let lang = if pokemon.is_legendary || &pokemon.habitat == "cave" {
                Language::Yoda
            } else {
                Language::Shakespear
            };

            let possible_translation = translation_api.translate(&pokemon.description, lang).await;

            if let Ok(translated) = possible_translation {
                pokemon.description = translated;
            }

            ok(pokemon)
        }
        // TODO: Surface more accurate errors
        Err(_) => not_found(format!("Unable to find '{}'", name)),
    }
}

pub(crate) fn rocket(settings: Settings) -> Rocket<Build> {
    let poke_api_client = settings.poke_api_client();
    let translation_client = settings.translation_api_client();

    rocket::build()
        .manage(poke_api_client)
        .manage(translation_client)
        .mount("/", rocket::routes![find_pokemon, find_translated_pokemon])
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::translation::Language;
    use assert_json_diff::assert_json_eq;
    use mocks::*;
    use rocket::http::Status;

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

    #[tokio::test]
    async fn requesting_mewtwo_makes_a_call_to_the_pokemon_api() {
        let (client, poke_mock, _) = setup().await;

        poke_mock.is_present("mewtwo", RAW_MEWTWO).await;

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
        let (client, poke_mock, _) = setup().await;

        poke_mock.no_pokemon_exist().await;

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
    async fn when_asking_for_a_cave_pokemon_the_translation_is_in_yoda_speak() {
        let (client, poke_mock, translation_mock) = setup().await;

        poke_mock.is_present("diglett", RAW_DIGLETT).await;
        translation_mock
            .can_translate(Language::Yoda, DIGLETT_AS_YODA)
            .await;

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

    #[tokio::test]
    async fn when_asking_for_a_legendary_pokemon_the_translation_is_in_yoda_speak() {
        let (client, poke_mock, translation_mock) = setup().await;

        // MewTwo is definitly legendary!
        poke_mock.is_present("mewtwo", RAW_MEWTWO).await;
        translation_mock
            .can_translate(Language::Yoda, MEWTWO_AS_YODA)
            .await;

        let response = client.get("/pokemon/translated/mewtwo").dispatch().await;
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
                    "description": "Created by a scientist after years of horrific gene splicing and dna engineering experiments,  it was.",
                    "habitat":"rare",
                    "isLegendary": true
                }
                "#
            )
        );
    }

    #[tokio::test]
    async fn non_legendary_or_cave_pokemon_are_translated_to_shakespearan_english() {
        let (client, poke_mock, translation_mock) = setup().await;

        // Just a plain bulbasaur
        poke_mock.is_present("bulbasaur", RAW_BULBASAUR).await;
        translation_mock
            .can_translate(Language::Shakespear, BULBASAUR_AS_SHAKESPEARE)
            .await;

        let response = client.get("/pokemon/translated/bulbasaur").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        let bulbasaur_json = response
            .into_string()
            .await
            .expect("Unexpected empty response");

        assert_json_eq!(
            json(&bulbasaur_json),
            json(
                r#"
                {
                    "name": "bulbasaur",
                    "description": "A strange seed wast planted on its back at birth. The plant sprouts and grows with this pokémon.",
                    "habitat":"grassland",
                    "isLegendary": false
                }
                "#
            )
        );
    }

    #[tokio::test]
    async fn when_the_translation_fails_we_fall_back_to_the_standard_description() {
        let (client, poke_mock, translation_mock) = setup().await;

        poke_mock.is_present("diglett", RAW_DIGLETT).await;
        translation_mock.fails_to_translate(Language::Yoda).await;

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
                    "description": "Lives about one yard underground where it feeds on plant roots. It sometimes appears above ground.",
                    "habitat":"cave",
                    "isLegendary":false
                }
                "#
            )
        );
    }

    pub mod mocks {
        use crate::rocket;
        use crate::translation::Language;
        use crate::{pokeapi::PokeApiSettings, translation::TranslationSettings, Settings};

        use rocket::local::asynchronous::Client;
        use wiremock::{
            matchers::{any, method, path},
            Mock, MockServer, ResponseTemplate,
        };

        pub const RAW_MEWTWO: &'static str = include_str!("../examples/pokeapi/mewtwo.json");
        pub const RAW_DIGLETT: &'static str = include_str!("../examples/pokeapi/diglett.json");
        pub const RAW_BULBASAUR: &'static str = include_str!("../examples/pokeapi/bulbasaur.json");

        pub const DIGLETT_AS_YODA: &'static str =
            include_str!("../examples/translation/diglett_yoda.json");
        pub const MEWTWO_AS_YODA: &'static str =
            include_str!("../examples/translation/mewtwo_yoda.json");
        pub const BULBASAUR_AS_SHAKESPEARE: &'static str =
            include_str!("../examples/translation/bulbasaur_shakespeare.json");

        pub async fn setup() -> (Client, MockPokeApi, MockTranslationApi) {
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

            let client = Client::tracked(rocket(settings)).await.unwrap();

            (
                client,
                MockPokeApi(poke_server),
                MockTranslationApi(translation_server),
            )
        }

        pub struct MockPokeApi(pub MockServer);

        impl MockPokeApi {
            pub async fn is_present(&self, pokemon: &'static str, response: &'static str) {
                let mock = Mock::given(method("GET"))
                    .and(path(format!("/api/v2/pokemon-species/{}", pokemon)))
                    .respond_with(
                        ResponseTemplate::new(200).set_body_raw(response, "application/json"),
                    )
                    .expect(1);

                self.0.register(mock).await;
            }

            pub async fn no_pokemon_exist(&self) {
                let mock = Mock::given(any())
                    .respond_with(ResponseTemplate::new(404))
                    .expect(1);

                self.0.register(mock).await;
            }
        }

        pub struct MockTranslationApi(pub MockServer);

        impl MockTranslationApi {
            pub(crate) async fn can_translate(&self, lang: Language, response: &'static str) {
                let mock = Mock::given(method("POST"))
                    .and(path(format!("/translate/{}", lang)))
                    .respond_with(
                        ResponseTemplate::new(200).set_body_raw(response, "application/json"),
                    )
                    .expect(1);

                self.0.register(mock).await;
            }

            pub(crate) async fn fails_to_translate(&self, lang: Language) {
                let mock = Mock::given(method("POST"))
                    .and(path(format!("/translate/{}", lang)))
                    .respond_with(ResponseTemplate::new(500))
                    .expect(1);

                self.0.register(mock).await;
            }
        }
    }
}
