use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};

lazy_static! {
    static ref RE_TOKEN: Regex = Regex::new(r"\w+").unwrap();
    static ref RE_SENTENCE: Regex = Regex::new(r"[.!?]+").unwrap();
}

pub fn show_text_insights(text: &str) {
    println!("\n📊 Text Insights 📊");
    println!("-----------------");

    let char_count = text.chars().count();
    println!("• Total characters: {}", char_count);

    let words: Vec<_> = RE_TOKEN.find_iter(text).collect();
    let word_count = words.len();
    println!("• Total words: {}", word_count);

    let unique_words: HashSet<_> = words.iter().map(|m| m.as_str().to_lowercase()).collect();
    println!(
        "• Unique words: {} ({:.1}% of total)",
        unique_words.len(),
        (unique_words.len() as f64 / word_count as f64) * 100.0
    );

    let sentences: Vec<_> = RE_SENTENCE.split(text).collect();
    let sentence_count = sentences.len();
    println!("• Estimated sentences: {}", sentence_count);

    if sentence_count > 0 {
        let avg_words_per_sentence = word_count as f64 / sentence_count as f64;
        println!(
            "• Average words per sentence: {:.1}",
            avg_words_per_sentence
        );
    }

    let mut word_counts: HashMap<String, usize> = HashMap::new();
    for word in words.iter() {
        let word_lower = word.as_str().to_lowercase();
        *word_counts.entry(word_lower).or_insert(0) += 1;
    }

    let mut word_freq: Vec<(String, usize)> = word_counts
        .iter()
        .filter(|(word, _)| word.len() > 2)
        .map(|(word, count)| (word.clone(), *count))
        .collect();

    word_freq.sort_by(|a, b| b.1.cmp(&a.1));

    println!("• Most common words:");
    for (i, (word, count)) in word_freq.iter().take(5).enumerate() {
        println!(
            "  {}. \"{}\" (appears {} times, {:.1}%)",
            i + 1,
            word,
            count,
            (*count as f64 / word_count as f64) * 100.0
        );
    }

    let ttr = unique_words.len() as f64 / word_count as f64;
    println!("• Vocabulary richness (TTR): {:.3}", ttr);

    let total_word_length: usize = words.iter().map(|m| m.as_str().len()).sum();
    let avg_word_length = total_word_length as f64 / word_count as f64;
    println!("• Average word length: {:.1} characters", avg_word_length);

    if word_count > 0 {
        let mut entropy = 0.0;
        for (_, count) in word_counts.iter() {
            let probability = *count as f64 / word_count as f64;
            entropy -= probability * probability.log2();
        }
        println!("• Shannon entropy: {:.3} bits", entropy);

        let perplexity = 2.0_f64.powf(entropy);
        println!("• Perplexity: {:.2}", perplexity);

        println!("  (Lower entropy/perplexity = more predictable text)");
    }

    println!();
}
