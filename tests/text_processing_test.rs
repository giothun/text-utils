use text_gen_ngram::text::processing::tokenize_large_text;
use text_gen_ngram::text::{TokenizerOptions, normalize_text, tokenize};

#[test]
fn test_normalize_text() {
    let text = "  Hello,   world!  ";
    let normalized = normalize_text(text);
    assert_eq!(normalized, "Hello, world!");

    let text = "Hello\n\nworld\t\ttest";
    let normalized = normalize_text(text);
    assert_eq!(normalized, "Hello world test");

    let text = "";
    let normalized = normalize_text(text);
    assert_eq!(normalized, "");

    let text = "   \t\n   ";
    let normalized = normalize_text(text);
    assert_eq!(normalized, "");
}

#[test]
fn test_tokenize_lowercase_option() {
    let text = "Hello, WORLD!";

    let options = TokenizerOptions::default();
    let tokens = tokenize(text, &options);
    assert!(tokens.contains(&"hello".to_string()));
    assert!(tokens.contains(&"world".to_string()));
    assert!(!tokens.contains(&"WORLD".to_string()));

    let options = TokenizerOptions {
        lowercase: false,
        ..Default::default()
    };
    let tokens = tokenize(text, &options);
    assert!(tokens.contains(&"Hello".to_string()));
    assert!(tokens.contains(&"WORLD".to_string()));
    assert!(!tokens.contains(&"world".to_string()));
}

#[test]
fn test_tokenize_punctuation_option() {
    let text = "Hello, world! This is a test.";

    let options = TokenizerOptions::default();
    let tokens = tokenize(text, &options);

    println!("Tokens with punctuation: {:?}", tokens);

    assert!(tokens.contains(&",".to_string()));
    assert!(tokens.contains(&"world!".to_string()));
    assert!(tokens.contains(&".".to_string()));

    let options = TokenizerOptions {
        preserve_punctuation: false,
        ..Default::default()
    };
    let tokens = tokenize(text, &options);

    println!("Tokens without punctuation: {:?}", tokens);

    assert!(!tokens.contains(&",".to_string()));
    assert!(!tokens.contains(&".".to_string()));
}

#[test]
fn test_tokenize_sentence_boundaries_option() {
    let text = "First. Second.";

    let options = TokenizerOptions::default();
    let tokens = tokenize(text, &options);
    assert!(
        tokens.contains(&"<sentence>".to_string()) || tokens.contains(&"<SENTENCE>".to_string())
    );

    let options = TokenizerOptions {
        preserve_sentence_boundaries: false,
        ..Default::default()
    };
    let tokens = tokenize(text, &options);
    assert!(
        !tokens.contains(&"<sentence>".to_string()) && !tokens.contains(&"<SENTENCE>".to_string())
    );
}

#[test]
fn test_tokenize_edge_cases() {
    let text = "";
    let options = TokenizerOptions::default();
    let tokens = tokenize(text, &options);
    assert!(tokens.is_empty());

    let text = ".,!?";
    let options = TokenizerOptions::default();
    let tokens = tokenize(text, &options);
    assert_eq!(tokens.len(), 4);

    let text = "   \t\n   ";
    let options = TokenizerOptions::default();
    let tokens = tokenize(text, &options);
    assert!(tokens.is_empty());
}

#[test]
fn test_tokenize_large_text() {
    let mut large_text = String::with_capacity(1_100_000);
    for _ in 0..110_000 {
        large_text.push_str("hello world ");
    }

    let options = TokenizerOptions::default();
    let result = tokenize_large_text(&large_text, &options);

    assert!(result.is_ok());
    let tokens = result.unwrap();
    assert!(!tokens.is_empty());
    assert!(tokens.contains(&"hello".to_string()));
    assert!(tokens.contains(&"world".to_string()));

    let small_text = "Hello, world!";
    let result = tokenize_large_text(small_text, &options);

    assert!(result.is_ok());
    let tokens = result.unwrap();
    assert_eq!(tokens, tokenize(small_text, &options));
}
