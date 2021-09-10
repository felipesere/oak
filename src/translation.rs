use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct Contents {
    text: String,
    translated: String,
}

#[derive(Debug, Deserialize)]
struct ExtendedTranslation {
    contents: Contents,
}

#[derive(Debug)]
pub(crate) struct TranslationClient {
    client: Client,
    domain: String,
}

pub(crate) struct TranslationSettings {
    pub(crate) base_url: String,
    pub(crate) timeout: Duration,
}

impl From<TranslationSettings> for TranslationClient {
    fn from(settings: TranslationSettings) -> Self {
        TranslationClient::new(settings.base_url, settings.timeout)
    }
}

enum Language {
    Yoda,
    Shakespear,
}

#[derive(Debug)]
enum Error {}

impl TranslationClient {
    fn new(domain: String, timeout: Duration) -> TranslationClient {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("failed to construct a viable PokeApi client");
        TranslationClient { client, domain }
    }

    async fn translate<S: Into<String>>(
        &self,
        text: S,
        _language: Language,
    ) -> Result<ExtendedTranslation, Error> {
        #[derive(Serialize)]
        struct Text {
            text: String,
        }

        let translation = self.client
            .post(format!("{}/translate/yoda", self.domain))
            .json(&Text { text: text.into() })
            .send()
            .await
            .expect("Failed to transfer request?")
            .json::<ExtendedTranslation>()
            .await
            .expect("Failed to turn translation to JSON");

        Ok(translation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const CONNECTION_TIMEOUT: Duration = Duration::from_millis(100);

    #[test]
    fn deserializes_a_successful_translation() {
        let yoda_json = r#"
            {
                "contents": {
                    "text": "This is fantastic",
                    "translated": "Fantastic,  this is",
                    "translation": "yoda"
                },
                "success": {
                    "total": 1
                }
            }
            "#;

        let extended_translation = serde_json::from_str::<ExtendedTranslation>(yoda_json)
            .expect("Unable to deserialize Yoda translation");

        assert_eq!(
            extended_translation.contents.text,
            "This is fantastic".to_string()
        );
        assert_eq!(
            extended_translation.contents.translated,
            "Fantastic,  this is".to_string()
        );
    }

    async fn setup() -> (TranslationClient, MockServer) {
        let translation_server = MockServer::start().await;

        let settings = TranslationSettings {
            base_url: format!("http://{}", translation_server.address().to_string()),
            timeout: CONNECTION_TIMEOUT,
        };

        (settings.into(), translation_server)
    }

    #[tokio::test]
    async fn translates_a_simple_sentence_to_yoda_speak() {
        let (client, mock_server) = setup().await;

        Mock::given(method("POST"))
            .and(path("/translate/yoda"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                include_str!("../examples/translation/yoda.json"),
                "application/json",
            ))
            .expect(1)
            .mount(&mock_server)
            .await;

        let yoda_translation = client
            .translate("This is fantastic", Language::Yoda)
            .await
            .expect("Unable to get translation");

        assert_eq!(
            yoda_translation.contents.translated,
            "Fantastic,  this is".to_string()
        );
    }
}
