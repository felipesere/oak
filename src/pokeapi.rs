use serde::Deserialize;
use thiserror::Error;
// TODO: Consider a custom deserializer and custom types

#[derive(Deserialize)]
struct Habitat {
    name: String,
}

#[derive(Deserialize)]
struct Language {
    name: String
}

#[derive(Deserialize)]
struct FlavorText {
    flavor_text: String,
    language: Language,
}

#[derive(Deserialize)]
struct Pokemon {
    name: String,
    is_legendary: bool,
    habitat: Habitat,
    flavor_text_entries: Vec<FlavorText>,
}

struct PokeClient {
}

#[derive(Error, Debug)]
enum Error {}

impl PokeClient {
    async fn find(&self, name: &str) -> Result<Pokemon, Error> {
        let pokemon = reqwest::get(format!("https://pokeapi.co/api/v2/pokemon-species/{}", name))
            .await
            .expect("Failed to make request to PokeApi")
            .json::<Pokemon>()
            .await
            .expect("Failed to parse response as a Pokemon");

        Ok(pokemon)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use super::*;

    #[test]
    fn deserializes_ditto() {
        let raw_ditto = include_str!("../examples/ditto.json");

        let ditto = serde_json::from_str::<Pokemon>(raw_ditto).expect("unable to deserialize ditto");

        assert_eq!(ditto.name, "ditto".to_string());
        assert!(!ditto.is_legendary);
        assert_eq!(ditto.habitat.name, "urban".to_string());
        assert_eq!(ditto.flavor_text_entries.len(), 134);

        let first_flavour_text = &ditto.flavor_text_entries[0];
        assert_eq!(first_flavour_text.flavor_text, "It can freely recombine its own cellular structure to\ntransform into other life-forms.".to_string());
        assert_eq!(first_flavour_text.language.name, "en".to_string());
    }

    #[tokio::test]
    async fn retrieves_diglett_from_pokeapi() {
        let client = PokeClient{};

        let diglett = client.find("diglett").await.expect("Failed to get diglett");

        assert_eq!(diglett.name, "diglett".to_string());
        assert_eq!(diglett.habitat.name, "cave".to_string());
        assert!(!diglett.is_legendary);
    }
}
