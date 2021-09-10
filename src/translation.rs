use serde::Deserialize;

#[warn(dead_code)]
#[derive(Deserialize)]
struct Contents {
    text: String,
    translated: String,
}

#[warn(dead_code)]
#[derive(Deserialize)]
struct ExtendedTranslation {
    contents: Contents,
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let extended_translation = serde_json::from_str::<ExtendedTranslation>(yoda_json).expect("Unable to deserialize Yoda translation");

        assert_eq!(extended_translation.contents.text, "This is fantastic".to_string());
        assert_eq!(extended_translation.contents.translated, "Fantastic,  this is".to_string());
    }
}
