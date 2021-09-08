use serde::Deserialize;
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
}
