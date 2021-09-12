use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

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

#[derive(Clone)]
pub(crate) struct TranslationSettings {
    pub(crate) base_url: String,
    pub(crate) timeout: Duration,
}

impl From<TranslationSettings> for TranslationClient {
    fn from(settings: TranslationSettings) -> Self {
        TranslationClient::new(settings.base_url, settings.timeout)
    }
}

pub(crate) enum Language {
    Yoda,
    Shakespear,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Yoda => write!(f, "yoda"),
            Language::Shakespear => write!(f, "shakespeare"),
        }
    }
}

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Hit the hourly rate limit when trying to translate")]
    RateLimitHit,
    #[error("Tried to deserialize invalid translation")]
    BadJson,
    #[error("Unexpected error from translation API")]
    Other(reqwest::Error),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if let Some(StatusCode::TOO_MANY_REQUESTS) = err.status() {
            Error::RateLimitHit
        } else if err.is_decode() {
            Error::BadJson
        } else {
            Error::Other(err)
        }
    }
}

impl TranslationClient {
    fn new(domain: String, timeout: Duration) -> TranslationClient {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("failed to construct a viable PokeApi client");
        TranslationClient { client, domain }
    }

    pub(crate) async fn translate<S: AsRef<str>>(
        &self,
        text: S,
        language: Language,
    ) -> Result<String, Error> {
        #[derive(Serialize)]
        struct Text<'a> {
            text: &'a str,
        }

        let translation = self
            .client
            .post(format!("{}/translate/{}", self.domain, language))
            .json(&Text {
                text: text.as_ref(),
            })
            .send()
            .await?
            .error_for_status()?
            .json::<ExtendedTranslation>()
            .await?;

        Ok(translation.contents.translated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks;
    use claim::assert_matches;
    use pretty_assertions::assert_eq;

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

    #[tokio::test]
    async fn translates_a_simple_sentence_to_yoda_speak() {
        let mock_server = mocks::setup_translation_api().await;

        mock_server
            .can_translate(
                Language::Yoda,
                include_str!("../examples/translation/yoda.json"),
            )
            .await;

        let yoda_translation = mock_server
            .client()
            .translate("This is fantastic", Language::Yoda)
            .await
            .expect("Unable to get translation");

        assert_eq!(yoda_translation, "Fantastic,  this is".to_string());
    }

    #[tokio::test]
    async fn translates_a_weird_sentence_to_shakespeare_english() {
        let mock_server = mocks::setup_translation_api().await;

        mock_server
            .can_translate(
                Language::Shakespear,
                include_str!("../examples/translation/shakespeare.json"),
            )
            .await;

        let shakespeare_translation = mock_server
            .client()
            .translate("Any sentence...", Language::Shakespear)
            .await
            .expect("Unable to get translation");

        assert_eq!(
            shakespeare_translation,
            "Thee did giveth mr. Tim a hearty meal, but unfortunately what did doth englut did maketh him kicketh the bucket.".to_string()
        );
    }

    #[tokio::test]
    async fn reports_an_error_when_rate_limit_has_been_hit() {
        let mock_server = mocks::setup_translation_api().await;

        mock_server.has_hit_rate_limit().await;

        let err = mock_server
            .client()
            .translate("This is fantastic", Language::Yoda)
            .await
            .expect_err("Request should have failed due to rate limiting");

        assert_matches!(err, Error::RateLimitHit)
    }

    #[tokio::test]
    async fn reports_an_error_for_bad_json() {
        let mock_server = mocks::setup_translation_api().await;

        mock_server.can_translate(Language::Yoda, "{ }").await;

        let err = mock_server
            .client()
            .translate("This is fantastic", Language::Yoda)
            .await
            .expect_err("Request should have failed due to bad JSON");

        assert_matches!(err, Error::BadJson)
    }
}
