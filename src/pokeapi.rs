use serde::Deserialize;
use reqwest::{Client, StatusCode};
use thiserror::Error;
// TODO: Consider a custom deserializer and custom types

#[derive(Deserialize, Debug)]
struct Habitat {
    name: String,
}

#[derive(Deserialize, Debug)]
struct Language {
    name: String,
}

#[derive(Deserialize, Debug)]
struct FlavorText {
    flavor_text: String,
    language: Language,
}

#[derive(Deserialize, Debug)]
struct Pokemon {
    name: String,
    is_legendary: bool,
    habitat: Habitat,
    flavor_text_entries: Vec<FlavorText>,
}

#[derive(Debug)]
struct PokeClient {
    client: Client,
    domain: String,
}

impl PokeClient {
    fn new(domain: String) -> PokeClient {
        let client = Client::new();
        PokeClient { client, domain }
    }
}

#[derive(Error, Debug)]
enum Error {
    #[error("Did not find '{}'", .0)]
    NoSuchPokemon(String),
    #[error("Failed to establish connection")]
    ConnectionError(#[from] reqwest::Error),
}

impl PokeClient {
    async fn find(&self, name: &str) -> Result<Pokemon, Error> {
        let pokemon =  self
            .client
            .get(format!("{}/api/v2/pokemon-species/{}", self.domain, name))
            .send()
            .await?
            .error_for_status()
            .map_err(|e| {
                match e.status() {
                    Some(StatusCode::NOT_FOUND) => Error::NoSuchPokemon(name.to_string()),
                    _ => Error::ConnectionError(e),
                }
            })?
            .json::<Pokemon>()
            .await
            .expect("Failed to parse response as a Pokemon");

        Ok(pokemon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use pretty_assertions::assert_eq;
    use wiremock::{
        matchers::{method, path, any},
        Mock, MockServer, ResponseTemplate,
    };

    const RAW_DITTO: &'static str = include_str!("../examples/ditto.json");

    #[test]
    fn deserializes_ditto() {
        let ditto =
            serde_json::from_str::<Pokemon>(RAW_DITTO).expect("unable to deserialize ditto");

        assert_eq!(ditto.name, "ditto".to_string());
        assert!(!ditto.is_legendary);
        assert_eq!(ditto.habitat.name, "urban".to_string());
        assert_eq!(ditto.flavor_text_entries.len(), 134);

        let first_flavour_text = &ditto.flavor_text_entries[0];
        assert_eq!(first_flavour_text.flavor_text, "It can freely recombine its own cellular structure to\ntransform into other life-forms.".to_string());
        assert_eq!(first_flavour_text.language.name, "en".to_string());
    }

    #[tokio::test]
    async fn retrieves_ditto_from_pokeapi() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/ditto"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(RAW_DITTO, "application/json"))
            .mount(&mock_server)
            .await;

        let client = PokeClient::new(format!("http://{}", mock_server.address()));

        let diglett = client.find("ditto").await.expect("Failed to get diglett");

        assert_eq!(diglett.name, "ditto".to_string());
        assert_eq!(diglett.habitat.name, "urban".to_string());
        assert!(!diglett.is_legendary);
    }

    #[tokio::test]
    async fn error_when_pokemon_isnt_real() {
        let mock_server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&mock_server)
            .await;

        let client = PokeClient::new(format!("http://{}", mock_server.address()));

        let err = client.find("not-a-pokemon").await.expect_err("should have failed to find 'not-a-pokemon'");

        assert_matches!(err, Error::NoSuchPokemon(_));
    }
}
