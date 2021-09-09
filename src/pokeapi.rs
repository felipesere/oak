use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

// TODO: Consider a custom deserializer and custom types
// TODO: Consider if I want to break this file up into several parts

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
pub(crate) struct Pokemon {
    name: String,
    is_legendary: bool,
    habitat: Habitat,
    flavor_text_entries: Vec<FlavorText>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct PokeApiSettings {
    pub(crate) base_url: String,
    #[serde(with = "humantime_serde")]
    pub(crate) timeout: Duration,
}

impl From<PokeApiSettings> for PokeClient {
    fn from(settings: PokeApiSettings) -> Self {
        PokeClient::new(settings.base_url, settings.timeout)
    }
}

#[derive(Debug)]
pub(crate) struct PokeClient {
    client: Client,
    domain: String,
}

#[allow(dead_code)]
#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Did not find '{}'", .pokemon)]
    NoSuchPokemon { pokemon: String },
    #[error("Received bad JSON from the server")]
    BadJson,
    #[error("Failed to establish connection")]
    Other(#[from] reqwest::Error),
}

impl PokeClient {
    pub(crate) fn new(domain: String, timeout: Duration) -> PokeClient {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("failed to construct a viable PokeApi client");
        PokeClient { client, domain }
    }

    #[allow(dead_code)]
    pub(crate) async fn find(&self, name: &str) -> Result<Pokemon, Error> {
        self.client
            .get(format!("{}/api/v2/pokemon-species/{}", self.domain, name))
            .send()
            .await?
            .error_for_status()
            .map_err(|e| match e.status() {
                Some(StatusCode::NOT_FOUND) => Error::NoSuchPokemon {
                    pokemon: name.to_string(),
                },
                _ => Error::Other(e),
            })?
            .json::<Pokemon>()
            .await
            .map_err(|e| {
                if e.is_decode() {
                    Error::BadJson
                } else {
                    Error::Other(e)
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use pretty_assertions::assert_eq;
    use std::time::Duration;
    use wiremock::{
        matchers::{any, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    const RAW_DITTO: &'static str = include_str!("../examples/ditto.json");
    const CONNECTION_TIMEOUT: Duration = Duration::from_millis(100);

    async fn setup() -> (PokeClient, MockServer) {
        let poke_server = MockServer::start().await;

        let settings = PokeApiSettings {
            base_url: format!("http://{}", poke_server.address().to_string()),
            timeout: CONNECTION_TIMEOUT,
        };

        (settings.into(), poke_server)
    }

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
        let (client, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/ditto"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(RAW_DITTO, "application/json"))
            .mount(&mock_server)
            .await;

        let ditto = client.find("ditto").await.expect("Failed to get ditto");

        assert_eq!(ditto.name, "ditto".to_string());
        assert_eq!(ditto.habitat.name, "urban".to_string());
        assert!(!ditto.is_legendary);
    }

    #[tokio::test]
    async fn error_when_pokemon_isnt_real() {
        let (client, mock_server) = setup().await;

        Mock::given(any())
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&mock_server)
            .await;

        let err = client
            .find("not-a-pokemon")
            .await
            .expect_err("should have failed to find 'not-a-pokemon'");

        assert_matches!(err, Error::NoSuchPokemon { .. });
    }

    #[tokio::test]
    async fn error_when_retrieving_ditto_takes_too_long() {
        let (client, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/ditto"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(RAW_DITTO, "application/json")
                    .set_delay(CONNECTION_TIMEOUT * 2),
            )
            .mount(&mock_server)
            .await;

        let err = client
            .find("ditto")
            .await
            .expect_err("should have failed with a timeout");

        assert_matches!(err, Error::Other(_));
    }

    #[tokio::test]
    async fn response_for_ditto_is_missing_some_values() {
        let (client, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/ditto"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(r#"{"name": "ditto"}"#, "application/json"),
            )
            .mount(&mock_server)
            .await;

        let err = client
            .find("ditto")
            .await
            .expect_err("should have failed due to bad json");

        assert_matches!(err, Error::BadJson { .. })
    }

    #[test]
    fn reads_configuration() {
        let pokeapi_yaml = r#"
            base_url: http://somewhere.com:123
            timeout: 15s
        "#;

        let pokeapi_settings = serde_yaml::from_str::<PokeApiSettings>(pokeapi_yaml)
            .expect("should have parsed PokeApi config YAML");

        assert_eq!(
            pokeapi_settings,
            PokeApiSettings {
                base_url: "http://somewhere.com:123".into(),
                timeout: Duration::from_secs(15),
            }
        );
    }
}
