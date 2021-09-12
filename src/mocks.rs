use crate::rocket;
use crate::translation::Language;
use crate::{
    pokeapi::{PokeApiSettings, PokeClient},
    translation::{TranslationClient, TranslationSettings},
    Settings,
};

use rocket::local::asynchronous::Client;
use std::time::Duration;
use wiremock::{
    matchers::{any, method, path},
    Mock, MockServer, ResponseTemplate,
};

pub const RAW_MEWTWO: &'static str = include_str!("../examples/pokeapi/mewtwo.json");
pub const RAW_DIGLETT: &'static str = include_str!("../examples/pokeapi/diglett.json");
pub const RAW_DITTO: &'static str = include_str!("../examples/pokeapi/ditto.json");
pub const RAW_BULBASAUR: &'static str = include_str!("../examples/pokeapi/bulbasaur.json");

pub const DIGLETT_AS_YODA: &'static str = include_str!("../examples/translation/diglett_yoda.json");
pub const MEWTWO_AS_YODA: &'static str = include_str!("../examples/translation/mewtwo_yoda.json");
pub const BULBASAUR_AS_SHAKESPEARE: &'static str =
    include_str!("../examples/translation/bulbasaur_shakespeare.json");

const CONNECTION_TIMEOUT: Duration = Duration::from_millis(100);

pub async fn setup_poke_api() -> MockPokeApi {
    let server = MockServer::start().await;
    let poke_api_settings = PokeApiSettings {
        base_url: format!("http://{}", server.address().to_string()),
        timeout: CONNECTION_TIMEOUT,
    };

    MockPokeApi {
        server,
        client: poke_api_settings.clone().into(),
        settings: poke_api_settings,
    }
}

pub async fn setup_translation_api() -> MockTranslationApi {
    let server = MockServer::start().await;
    let translation_api_settings = TranslationSettings {
        base_url: format!("http://{}", server.address().to_string()),
        timeout: CONNECTION_TIMEOUT,
    };

    MockTranslationApi {
        server,
        client: translation_api_settings.clone().into(),
        settings: translation_api_settings,
    }
}

pub async fn setup() -> (Client, MockPokeApi, MockTranslationApi) {
    let mock_poke_api = setup_poke_api().await;
    let mock_translation_api = setup_translation_api().await;

    let settings = Settings {
        poke_api: mock_poke_api.settings.clone(),
        translation_api: mock_translation_api.settings.clone(),
    };

    let client = Client::tracked(rocket(settings)).await.unwrap();

    (client, mock_poke_api, mock_translation_api)
}

pub struct MockPokeApi {
    server: MockServer,
    client: PokeClient,
    settings: PokeApiSettings,
}

impl MockPokeApi {
    pub async fn is_present(&self, pokemon: &'static str, response: &'static str) {
        let mock = Mock::given(method("GET"))
            .and(path(format!("/api/v2/pokemon-species/{}", pokemon)))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response, "application/json"))
            .expect(1);

        self.server.register(mock).await;
    }

    pub async fn is_slow_to_respond(&self, pokemon: &'static str) {
        let mock = Mock::given(method("GET"))
            .and(path(format!("/api/v2/pokemon-species/{}", pokemon)))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(RAW_DITTO, "application/json")
                    .set_delay(CONNECTION_TIMEOUT * 2),
            )
            .expect(1);

        self.server.register(mock).await;
    }

    pub async fn no_pokemon_exist(&self) {
        let mock = Mock::given(any())
            .respond_with(ResponseTemplate::new(404))
            .expect(1);

        self.server.register(mock).await;
    }

    pub(crate) fn client(&self) -> &PokeClient {
        &self.client
    }
}

pub struct MockTranslationApi {
    server: MockServer,
    client: TranslationClient,
    settings: TranslationSettings,
}

impl MockTranslationApi {
    pub(crate) async fn can_translate(&self, lang: Language, response: &'static str) {
        let mock = Mock::given(method("POST"))
            .and(path(format!("/translate/{}", lang)))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response, "application/json"))
            .expect(1);

        self.server.register(mock).await;
    }

    pub(crate) async fn has_hit_rate_limit(&self) {
        let mock = Mock::given(method("POST"))
            .and(path("/translate/yoda"))
            .respond_with(ResponseTemplate::new(429).set_body_raw(
                r#"{
                    "error": {
                        "code": 429,
                        "message": "Too Many Requests: Rate limit of 5 requests per hour exceeded. Please wait for 17 minutes and 41 seconds."
                    }
                }"#,
                "application/json",
            ))
            .expect(1);

        self.server.register(mock).await;
    }

    pub(crate) async fn fails_to_translate(&self, lang: Language) {
        let mock = Mock::given(method("POST"))
            .and(path(format!("/translate/{}", lang)))
            .respond_with(ResponseTemplate::new(500))
            .expect(1);

        self.server.register(mock).await;
    }

    pub(crate) fn client(&self) -> &TranslationClient {
        &self.client
    }
}
