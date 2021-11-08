use std::time::Duration;

use pokeapi::{PokeApiSettings, PokeClient};
use server::rocket;
use translation::{TranslationClient, TranslationSettings};

use serde::Deserialize;

mod pokeapi;
mod server;
mod translation;

#[cfg(test)]
mod mocks;

#[derive(Debug, Deserialize)]
struct Settings {
    poke_api: PokeApiSettings,
    translation_api: TranslationSettings,
}

impl Settings {
    fn from_env() -> Self {
        let poke_api_base_url = env_var("APP_POKE_API_BASE_URL");
        let poke_api_timeout = env_var("APP_POKE_API_TIMEOUT");

        let translation_api_base_url = env_var("APP_TRANSLATION_API_BASE_URL");
        let translation_api_timeout = env_var("APP_TRANSLATION_API_TIMEOUT");

        Settings {
            poke_api: PokeApiSettings {
                base_url: poke_api_base_url,
                timeout: parse(poke_api_timeout).unwrap(),
            },
            translation_api: TranslationSettings {
                base_url: translation_api_base_url,
                timeout: parse(translation_api_timeout).unwrap(),
            },
        }
    }

    fn poke_api_client(&self) -> PokeClient {
        self.poke_api.clone().into()
    }

    fn translation_api_client(&self) -> TranslationClient {
        self.translation_api.clone().into()
    }
}

fn parse(input: String) -> Result<Duration, String> {
    input
        .parse::<humantime::Duration>()
        .map(Into::into)
        .map_err(|e| format!("{}", e))
}

fn env_var(name: &'static str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{} not present", name))
}

#[rocket::main]
async fn main() {
    let settings = Settings::from_env();
    let _ = rocket(settings).launch().await;
}
