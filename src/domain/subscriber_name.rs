use unicode_segmentation::UnicodeSegmentation;


#[derive(Debug, serde::Serialize)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(name: String) -> Result<Self, String> {
        let is_empty_or_whitespace = name.trim().is_empty();

        // A grapheme is defined by the Unicode standard as a "user-perceived"
        // character: `å` is a single grapheme, but it is composed of two characters
        // (`a` and `̊`).
        //
        // `graphemes` returns an iterator over the graphemes in the input `s`.
        // `true` specifies that we want to use the extended grapheme definition set,
        // the recommended one.
        let is_too_long = name.graphemes(true).count() > 256;

        // Iterate over all characters in the input to check if any of them is matches the forbidden characters
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = name.chars().any(|c| forbidden_characters.contains(&c));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid subscriber name", name))
        } else {
            Ok(Self(name))
        }
    }

    pub fn inner(self) -> String {
        self.0
    }

    pub fn inner_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}


/// Returns true if the input satisfies all our validation constraints on subscriber's name
pub fn is_valid_name(name: &str) -> bool {
    let is_empty_or_whitespace = name.trim().is_empty();

    // A grapheme is defined by the Unicode standard as a "user-perceived"
    // character: `å` is a single grapheme, but it is composed of two characters
    // (`a` and `̊`).
    //
    // `graphemes` returns an iterator over the graphemes in the input `s`.
    // `true` specifies that we want to use the extended grapheme definition set,
    // the recommended one.
    let is_too_long = name.graphemes(true).count() > 256;

    // Iterate over all characters in the input to check if any of them is matches the forbidden charaters
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = name.chars().any(|c| forbidden_characters.contains(&c));

    !(is_empty_or_whitespace || is_too_long || contains_forbidden_characters)
}

#[cfg(test)]
mod tests {
    use crate::domain::subscriber_name::SubscriberName;
    use claim::{assert_err, assert_ok};

    #[test]
    fn test_a_256_grapheme_long_name_is_valid() {
        let name = "a".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }
    #[test]
    fn test_whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }
    #[test]
    fn test_empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }
    #[test]
    fn test_a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }
    #[test]
    fn test_names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = format!("valid_name_{}", name);
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn test_a_valid_name_is_parsed_successfully() {
        let name = "Hellsent Alfather".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}