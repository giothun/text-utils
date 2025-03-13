use crate::error::{Result, TextGenError};
use once_cell::sync::Lazy;
use regex::Regex;

pub struct TokenizerOptions {
    /// Whether to convert text to lowercase
    ///
    /// Setting this to true reduces the vocabulary size and improves pattern recognition,
    /// but may affect the capitalization in the generated text.
    pub lowercase: bool,

    /// Whether to preserve punctuation as separate tokens
    ///
    /// Setting this to true helps maintain sentence structure and readability
    /// in the generated text, but increases the vocabulary size.
    pub preserve_punctuation: bool,

    /// Whether to preserve sentence boundaries
    ///
    /// Setting this to true prevents the model from generating nonsensical
    /// transitions between sentences, but may make the model more rigid.
    pub preserve_sentence_boundaries: bool,
}

impl Default for TokenizerOptions {
    fn default() -> Self {
        Self {
            lowercase: true,
            preserve_punctuation: true,
            preserve_sentence_boundaries: true,
        }
    }
}

static SENTENCE_BOUNDARY: Lazy<Regex> = Lazy::new(|| Regex::new("([.!?])\\s+([A-Z])").unwrap());

static PUNCTUATION: Lazy<Regex> = Lazy::new(|| Regex::new("([.,!?;:()\\[\\]{}\"'\\-])").unwrap());

static WHITESPACE: Lazy<Regex> = Lazy::new(|| Regex::new("\\s+").unwrap());

static TOKENIZER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("([.!?])\\s+([A-Z])|([.,!?;:()\\[\\]{}\"'\\-])|\\s+").unwrap());

pub fn tokenize(text: &str, options: &TokenizerOptions) -> Vec<String> {
    let estimated_token_count = text.len() / 5;
    let mut tokens = Vec::with_capacity(estimated_token_count);

    let processed_text = TOKENIZER_REGEX.replace_all(text, |caps: &regex::Captures| {
        if let (Some(end_punct), Some(start_char)) = (caps.get(1), caps.get(2)) {
            if options.preserve_sentence_boundaries {
                return format!("{} <SENTENCE> {}", end_punct.as_str(), start_char.as_str());
            } else {
                return format!("{} {}", end_punct.as_str(), start_char.as_str());
            }
        }

        if let Some(punct) = caps.get(3) {
            if options.preserve_punctuation {
                return format!(" {} ", punct.as_str());
            } else {
                return " ".to_string();
            }
        }

        " ".to_string()
    });

    for token in processed_text.split_whitespace() {
        if !token.is_empty() {
            if options.lowercase {
                tokens.push(token.to_lowercase());
            } else {
                tokens.push(token.to_string());
            }
        }
    }

    tokens
}

pub fn normalize_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut last_was_whitespace = false;

    for c in text.chars() {
        if c.is_whitespace() {
            if !last_was_whitespace {
                result.push(' ');
                last_was_whitespace = true;
            }
        } else {
            result.push(c);
            last_was_whitespace = false;
        }
    }

    if result.starts_with(' ') {
        result.remove(0);
    }
    if result.ends_with(' ') {
        result.pop();
    }

    result
}

pub fn tokenize_large_text(text: &str, options: &TokenizerOptions) -> Result<Vec<String>> {
    if text.len() > 1_000_000 {
        const CHUNK_SIZE: usize = 100_000;
        let mut all_tokens = Vec::with_capacity(text.len() / 5);

        for chunk in text.as_bytes().chunks(CHUNK_SIZE) {
            let chunk_str = match std::str::from_utf8(chunk) {
                Ok(s) => s,
                Err(_) => {
                    return Err(TextGenError::Tokenization(
                        "Invalid UTF-8 sequence in text".to_string(),
                    ));
                }
            };

            let chunk_tokens = tokenize(chunk_str, options);
            all_tokens.extend(chunk_tokens);
        }

        Ok(all_tokens)
    } else {
        Ok(tokenize(text, options))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_with_default_options() {
        let text = "Hello, world! This is a test.";
        let options = TokenizerOptions::default();
        let tokens = tokenize(text, &options);

        let expected = vec![
            "hello",
            ",",
            "world!",
            "<sentence>",
            "this",
            "is",
            "a",
            "test",
            ".",
        ];

        assert_eq!(
            tokens,
            expected.iter().map(|s| s.to_string()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_tokenize_without_punctuation() {
        let text = "Hello, world! This is a test.";
        let options = TokenizerOptions {
            preserve_punctuation: false,
            ..Default::default()
        };
        let tokens = tokenize(text, &options);

        let expected = vec!["hello", "world!", "<sentence>", "this", "is", "a", "test"];

        assert_eq!(
            tokens,
            expected.iter().map(|s| s.to_string()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_tokenize_with_sentence_boundaries() {
        let text = "First sentence. Second sentence! Third sentence?";
        let options = TokenizerOptions::default();
        let tokens = tokenize(text, &options);

        assert!(tokens.contains(&"<sentence>".to_string()));
    }
}
