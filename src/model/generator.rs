use super::ngram::NGramModel;
use log::debug;
use rand::seq::IteratorRandom;
use rand::{rng, rngs::ThreadRng};

pub struct Generator<'a> {
    model: &'a NGramModel,
}

impl<'a> Generator<'a> {
    pub fn new(model: &'a NGramModel) -> Self {
        Self { model }
    }

    pub fn generate(&self, seed: Option<Vec<String>>, length: usize) -> String {
        let mut rng: ThreadRng = rng();

        let (mut result, mut current_key) = match seed {
            Some(seed_words) => {
                let result = seed_words.clone();

                let context = if seed_words.len() >= self.model.n {
                    seed_words[seed_words.len() - self.model.n..].to_vec()
                } else {
                    let mut random_context = self
                        .model
                        .model
                        .keys()
                        .choose(&mut rng)
                        .expect("Model has no keys")
                        .clone();

                    let start_idx = self.model.n - seed_words.len();
                    for (i, word) in seed_words.iter().enumerate() {
                        random_context[start_idx + i] = word.clone();
                    }
                    random_context
                };

                (result, context)
            }
            None => {
                let context = self
                    .model
                    .model
                    .keys()
                    .choose(&mut rng)
                    .expect("Model has no keys")
                    .clone();

                (context.clone(), context)
            }
        };

        let mut tokens_generated = 0;
        while tokens_generated < length {
            if let Some(next_word) = self.model.predict_next(&current_key, &mut rng) {
                result.push(next_word.clone());

                current_key.remove(0);
                current_key.push(next_word);

                tokens_generated += 1;
            } else {
                debug!("No continuation found for context: {:?}", current_key);

                if self.model.model.is_empty() {
                    debug!("Model is empty, stopping generation");
                    break;
                }

                if let Some(new_context) = self.model.model.keys().choose(&mut rng) {
                    debug!("Switching to new random context: {:?}", new_context);
                    current_key = new_context.clone();
                } else {
                    debug!("No contexts available in model, stopping generation");
                    break;
                }
            }
        }

        self.format_generated_text(&result)
    }

    fn format_generated_text(&self, tokens: &[String]) -> String {
        let mut formatted = String::new();
        let mut capitalize_next = true;
        let mut prev_token = "";

        for (i, token) in tokens.iter().enumerate() {
            // Handle section headers (# symbol)
            if token == "#" && i < tokens.len() - 1 {
                if !formatted.is_empty() && !formatted.ends_with('\n') {
                    formatted.push_str("\n\n");
                }
                capitalize_next = true;
                continue;
            }

            if token == "<SENTENCE>" {
                // Replace sentence boundary marker with period
                if !formatted.ends_with('.')
                    && !formatted.ends_with('!')
                    && !formatted.ends_with('?')
                {
                    formatted.push('.');
                }
                formatted.push(' ');
                capitalize_next = true;
                continue;
            }

            // Check if token is punctuation
            let is_punct = token.len() == 1 && ",.!?;:()[]{}\"'".contains(token);

            // Special handling for apostrophes in contractions
            if token == "'" && i > 0 && i < tokens.len() - 1 {
                let next_token = &tokens[i + 1];
                // Check if this is likely a contraction (e.g., don't, can't, etc.)
                if ["t", "s", "ll", "ve", "re", "d", "m"].contains(&next_token.as_str()) {
                    formatted.push('\'');
                    prev_token = token;
                    continue;
                }
            }

            if is_punct {
                // No space before punctuation
                formatted.push_str(token);
                // Add space after punctuation unless it's opening bracket or quote
                if !["(", "[", "{", "\"", "'"].contains(&token.as_str()) {
                    formatted.push(' ');
                }
            } else {
                // For normal words
                // Don't add space after an apostrophe for contractions
                if i > 0
                    && !formatted.ends_with(' ')
                    && !formatted.is_empty()
                    && prev_token != "'"
                    && !formatted.ends_with('\n')
                {
                    formatted.push(' ');
                }

                // Capitalize first word or after sentence boundary
                if capitalize_next {
                    if let Some(c) = token.chars().next() {
                        let capitalized =
                            c.to_uppercase().collect::<String>() + &token[c.len_utf8()..];
                        formatted.push_str(&capitalized);
                    } else {
                        formatted.push_str(token);
                    }
                    capitalize_next = false;
                } else {
                    formatted.push_str(token);
                }
            }

            prev_token = token;
        }

        // Ensure the text ends with proper punctuation
        if !formatted.ends_with('.') && !formatted.ends_with('!') && !formatted.ends_with('?') {
            formatted.push('.');
        }

        // Clean up any remaining <SENTENCE> tags
        let cleaned = formatted.replace("<SENTENCE>", ". ");

        // Fix spacing issues
        let cleaned = cleaned
            .replace(" .", ".")
            .replace(" ,", ",")
            .replace(" !", "!")
            .replace(" ?", "?");

        cleaned.trim().to_string()
    }
}
