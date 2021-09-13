use crate::pokeapi::{Error, PokeClient};
use crate::translation::{Language, TranslationClient};
use crate::Settings;

use rocket::http::Status;
use rocket::Request;
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

fn internal_server_error<T>() -> ApiResult<T> {
    Result::Err((
        Status::InternalServerError,
        Json(ApiError {
            message: "Internal server error".into(),
        }),
    ))
}

#[rocket::get("/pokemon/<name>")]
async fn find_pokemon(poke_api: &State<PokeClient>, name: &str) -> ApiResult<Pokemon> {
    match poke_api.find(name).await {
        Ok(pokemon) => ok(pokemon),
        Err(Error::NoSuchPokemon) => {
            log::info!("Did not find a pokemon called '{}'", name);
            not_found(format!("Unable to find '{}'", name))
        }
        Err(error) => {
            log::info!("Error when looking for '{}': {}", name, error);
            internal_server_error()
        }
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
            log::info!("Using the '{}' translation for {}", &lang, pokemon.name);

            let possible_translation = translation_api.translate(&pokemon.description, lang).await;

            match possible_translation {
                Ok(translated) => {
                    log::info!("Successfull translation for {}", pokemon.name);
                    pokemon.description = translated;
                }
                Err(error) => {
                    log::error!(
                        "Failed to retrieve translation, falling back to standard text: {}",
                        error
                    );
                }
            }

            ok(pokemon)
        }
        Err(Error::NoSuchPokemon) => {
            log::info!("Did not find a pokemon called '{}'", name);
            not_found(format!("Unable to find '{}'", name))
        }
        Err(error) => {
            log::info!("Error when looking for '{}': {}", name, error);
            internal_server_error()
        }
    }
}

#[derive(Serialize)]
struct Examples {
    mewtwo: &'static str,
    diglett_translated: &'static str,
}

#[derive(Serialize)]
struct HelpMessage {
    message: String,
    help: &'static str,
    examples: Examples,
}

#[rocket::catch(404)]
fn help_message(req: &Request) -> Json<HelpMessage> {
    Json(HelpMessage {
        message: format!("Route '{}' was not found", req.uri().path()),
        help: "There are only two valid routes: '/pokemon/<name>' and '/pokemon/translated/<name>'",
        examples: Examples {
            mewtwo: "/pokemon/mewtwo",
            diglett_translated: "/pokemon/translated/diglett",
        },
    })
}

pub(crate) fn rocket(settings: Settings) -> Rocket<Build> {
    let poke_api_client = settings.poke_api_client();
    let translation_client = settings.translation_api_client();

    rocket::build()
        .register("/", rocket::catchers![help_message])
        .manage(poke_api_client)
        .manage(translation_client)
        .mount("/", rocket::routes![find_pokemon, find_translated_pokemon])
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mocks::*;
    use crate::translation::Language;
    use assert_json_diff::assert_json_eq;
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
    #[tokio::test]
    async fn requesting_non_existing_routes_gives_a_helpful_message_with_examples() {
        let (client, _, _) = setup().await;

        let response = client.get("/a/random/route").dispatch().await;

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
                    "message": "Route '/a/random/route' was not found",
                    "help": "There are only two valid routes: '/pokemon/<name>' and '/pokemon/translated/<name>'",
                    "examples": {
                        "mewtwo": "/pokemon/mewtwo",
                        "diglett_translated": "/pokemon/translated/diglett"
                    }
                }
                "#
            )
        );
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
    async fn lets_users_know_when_pokemon_were_not_found() {
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
    async fn other_errors_result_in_a_500_error() {
        let (client, poke_mock, _) = setup().await;

        poke_mock.is_present("mewtwo", "{not_even_json}").await;

        let response = client.get("/pokemon/mewtwo").dispatch().await;
        assert_eq!(response.status(), Status::InternalServerError);

        let error = response
            .into_string()
            .await
            .expect("Unexpected empty response");

        assert_json_eq!(
            json(&error),
            json(r#"{"message": "Internal server error"}"#)
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
    async fn when_the_translation_fails_it_falls_back_to_the_standard_description() {
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

    #[tokio::test]
    async fn when_translated_pokemon_does_not_exist_a_404_is_returned() {
        let (client, poke_mock, _) = setup().await;

        poke_mock.no_pokemon_exist().await;

        let response = client.get("/pokemon/translated/diglett").dispatch().await;
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
                    "message": "Unable to find 'diglett'"
                }
                "#
            )
        );
    }

    #[tokio::test]
    async fn when_requesting_a_translated_pokemon_fails() {
        let (client, poke_mock, _) = setup().await;

        poke_mock
            .is_present("diglett", r#"{not_really_even_json}"#)
            .await;

        let response = client.get("/pokemon/translated/diglett").dispatch().await;
        assert_eq!(response.status(), Status::InternalServerError);

        let error = response
            .into_string()
            .await
            .expect("Unexpected empty response");

        assert_json_eq!(
            json(&error),
            json(r#"{"message": "Internal server error"}"#)
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
