use pokeapi::{PokeApiSettings, PokeClient};
use translation::{TranslationClient, TranslationSettings};

use server::rocket;

mod pokeapi;
mod translation;
mod server;

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
