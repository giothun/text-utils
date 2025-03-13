use text_gen_ngram::{
    model::{Generator, NGramModel, Trainer},
    text::{TokenizerOptions, tokenize},
};

#[test]
fn test_end_to_end_generation() {
    let text = "The quick brown fox jumps over the lazy dog. \
                The fox is quick and brown. \
                The dog is lazy and sleeps all day.";

    let trainer = Trainer::new(2);

    let model = trainer.train_from_text(text).unwrap();

    let generator = Generator::new(&model);

    let seed = Some(vec!["The".to_string(), "fox".to_string()]);
    let generated = generator.generate(seed, 10);

    assert!(generated.starts_with("The fox"));

    assert!(!generated.is_empty());
}

#[test]
fn test_model_optimization() {
    let mut model = NGramModel::new(1);

    let tokens = vec![
        "a".to_string(),
        "b".to_string(),
        "a".to_string(),
        "b".to_string(),
        "a".to_string(),
        "c".to_string(),
        "a".to_string(),
        "c".to_string(),
        "a".to_string(),
        "c".to_string(),
        "a".to_string(),
        "c".to_string(),
    ];

    model.add_tokens(&tokens);

    let initial_total_tokens = model.get_stats().total_tokens;

    model.optimize();

    let optimized_stats = model.get_stats();

    assert_eq!(initial_total_tokens, optimized_stats.total_tokens);
}

#[test]
fn test_tokenization_options() {
    let text = "Hello, world! This is a test.";

    let default_options = TokenizerOptions::default();
    let default_tokens = tokenize(text, &default_options);

    let custom_options = TokenizerOptions {
        lowercase: false,
        preserve_punctuation: false,
        preserve_sentence_boundaries: false,
    };
    let custom_tokens = tokenize(text, &custom_options);

    assert!(default_tokens.contains(&",".to_string()));
    assert!(default_tokens.contains(&"hello".to_string()));

    assert!(!custom_tokens.contains(&",".to_string()));
    assert!(custom_tokens.contains(&"Hello".to_string()));
}
