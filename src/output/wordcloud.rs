use crate::error::{Result as TextGenResult, TextGenError};
use chrono::Utc;
use log;
use regex::Regex;
use std::collections::HashMap;
use wordcloud_rs::*;

pub fn generate_wordcloud(text: &str) -> TextGenResult<String> {
    let original_level = log::max_level();
    log::set_max_level(log::LevelFilter::Error);

    let tokens = prepare_tokens_for_wordcloud(text);

    let wc = WordCloud::new().generate(tokens);

    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let output_path = format!("wordcloud_{}.png", timestamp);

    let result = wc
        .save(&output_path)
        .map_err(|e| TextGenError::Image(e.to_string()));

    log::set_max_level(original_level);

    result.map(|_| output_path)
}

fn prepare_tokens_for_wordcloud(text: &str) -> Vec<(Token, f32)> {
    let re = Regex::new(r"\w+").unwrap();
    let tokens: Vec<String> = re
        .find_iter(text)
        .map(|m| m.as_str().to_lowercase())
        .collect();

    let mut counts: HashMap<String, usize> = HashMap::new();
    for word in tokens {
        if word.len() > 2 {
            *counts.entry(word).or_default() += 1;
        }
    }

    let mut wordcloud_tokens: Vec<(Token, f32)> = counts
        .into_iter()
        .map(|(k, v)| (Token::Text(k), v as f32))
        .collect();

    wordcloud_tokens.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    wordcloud_tokens.truncate(100);

    wordcloud_tokens
}
