use argh::FromArgs;
use pokeapi::{PokeApiSettings, PokeClient};
use server::rocket;
use translation::{TranslationClient, TranslationSettings};

use serde::Deserialize;

mod pokeapi;
mod server;
mod translation;

#[cfg(test)]
mod mocks;

#[derive(FromArgs)]
/// A simple Pokemon server that gives minimal information
struct Cli {
    #[argh(option)]
    /// from where to load additional config
    config: String,
}

#[derive(Debug, Deserialize)]
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
    let cli: Cli = argh::from_env();
    let settings_file = std::fs::read_to_string(cli.config).expect("should have read a config");
    let settings: Settings =
        serde_yaml::from_str(&settings_file).expect("Should have parsed config");

    let _ = rocket(settings).launch().await;
}
