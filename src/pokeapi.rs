use reqwest::{Client, StatusCode};
use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

use crate::server::Pokemon;

const FORM_FEED: char = '\u{c}';

#[derive(Deserialize, Debug)]
struct Habitat {
    name: String,
}

fn deserialize_flavour_text<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct FlavourTextVisitor;

    impl<'de> Visitor<'de> for FlavourTextVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("expected an array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            #[derive(Deserialize)]
            struct Language {
                name: String,
            }

            #[derive(Deserialize)]
            struct FlavorText {
                flavor_text: String,
                language: Language,
            }

            let mut found = None;
            while let Some(flavour_entry) = seq.next_element::<FlavorText>()? {
                if flavour_entry.language.name == "en" {
                    found = Some(flavour_entry.flavor_text);
                    break;
                }
            }
            // Consume any remaining flavour text entries without actually allocating any memory.
            // Serde requires that we consume the entire available JSON, probably to know where to
            // continue deserializing other fields?
            while let Some(_ignored_any) = seq.next_element::<IgnoredAny>()? {}

            found.ok_or_else(|| {
                serde::de::Error::custom("did not find an english variant of the flavour text")
            })
        }
    }

    deserializer.deserialize_seq(FlavourTextVisitor)
}

#[derive(Deserialize, Debug)]
struct ExternalPokemon {
    name: String,
    is_legendary: bool,
    habitat: Habitat,
    #[serde(
        rename = "flavor_text_entries",
        deserialize_with = "deserialize_flavour_text"
    )]
    description: String,
}

fn clean_text(input: &str) -> String {
    input.replace(&['\n', FORM_FEED][..], " ")
}

impl From<ExternalPokemon> for Pokemon {
    fn from(api_pokemon: ExternalPokemon) -> Self {
        let description = clean_text(&api_pokemon.description);

        Pokemon {
            name: api_pokemon.name,
            description,
            habitat: api_pokemon.habitat.name,
            is_legendary: api_pokemon.is_legendary,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
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
        log::info!("Getting information about {}", name);

        self.client
            .get(format!("{}/api/v2/pokemon-species/{}", self.domain, name))
            .send()
            .await?
            .error_for_status()
            .map_err(|e| match e.status() {
                Some(StatusCode::NOT_FOUND) => {
                    log::error!("Did not find {} on the PokeApi", name);
                    Error::NoSuchPokemon {
                        pokemon: name.to_string(),
                    }
                }
                _ => {
                    log::error!(
                        "Unexpected error when getting respose from PokeApi for {}: {}",
                        name,
                        e
                    );
                    Error::Other(e)
                }
            })?
            .json::<ExternalPokemon>()
            .await
            .map_err(|e| {
                if e.is_decode() {
                    log::error!("Received bad JSON for {}: {}", name, e);
                    Error::BadJson
                } else {
                    log::error!("Unexpected deserializing JSON for {}: {}", name, e);
                    Error::Other(e)
                }
            })
            .map(Pokemon::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::{assert_err, assert_matches};
    use pretty_assertions::assert_eq;
    use std::time::Duration;
    use wiremock::{
        matchers::{any, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    const RAW_DITTO: &'static str = include_str!("../examples/pokeapi/ditto.json");
    const RAW_MEWTWO: &'static str = include_str!("../examples/pokeapi/mewtwo.json");
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
        let ditto = serde_json::from_str::<ExternalPokemon>(RAW_DITTO)
            .expect("unable to deserialize ditto");

        assert_eq!(ditto.name, "ditto".to_string());
        assert!(!ditto.is_legendary);
        assert_eq!(ditto.habitat.name, "urban".to_string());
        assert_eq!(ditto.description, "It can freely recombine its own cellular structure to\ntransform into other life-forms.".to_string());
    }

    #[test]
    fn fails_if_there_is_no_english_flavour_text() {
        // Shrunk example with the necessary fields, except the only flavor text is german...
        let german_ditton = r#"
{
  "flavor_text_entries": [
     {
      "flavor_text": "Es kann seine Zellstruktur...",
      "language": {
        "name": "de"
      }
    }
  ],
  "habitat": {
    "name": "urban",
    "url": "https://pokeapi.co/api/v2/pokemon-habitat/8/"
  },
  "is_legendary": false,
  "name": "ditto"
}
            "#;
        let not_a_ditto = serde_json::from_str::<ExternalPokemon>(german_ditton);
        assert_err!(not_a_ditto);
    }

    #[test]
    fn cleanup_any_line_and_form_feed_characters_from_flavour_text() {
        // Rust can't represent \f in a literal, see examples/mewtwo.json
        // for more examples of the form feed
        let flavor_text = "Its DNA is almost\nthe same as MEW's.\nHowever, its size\u{000C}and disposition\nare vastly dif­\nferent.";

        let clean = r#"Its DNA is almost the same as MEW's. However, its size and disposition are vastly dif­ ferent."#.to_string();

        assert_eq!(clean_text(flavor_text), clean)
    }

    #[tokio::test]
    async fn retrieves_mewtwo_from_pokeapi() {
        let (client, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/mewtwo"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(RAW_MEWTWO, "application/json"))
            .mount(&mock_server)
            .await;

        let mewtwo = client.find("mewtwo").await.expect("Failed to get ditto");

        assert_eq!(mewtwo.name, "mewtwo".to_string());
        assert_eq!(mewtwo.habitat, "rare".to_string());
        assert_eq!(mewtwo.description, "It was created by a scientist after years of horrific gene splicing and DNA engineering experiments.".to_string());
        assert!(mewtwo.is_legendary);
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
