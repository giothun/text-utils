use rand::rng;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct NGramModel {
    pub n: usize,
    pub model: HashMap<Vec<String>, Vec<String>>,
    #[serde(skip)]
    pub stats: ModelStats,
}

#[derive(Default)]
pub struct ModelStats {
    pub total_tokens: usize,
    pub unique_contexts: usize,
    pub unique_continuations: usize,
    pub largest_continuation_set: usize,
}

impl NGramModel {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            model: HashMap::new(),
            stats: ModelStats::default(),
        }
    }

    pub fn add_tokens(&mut self, tokens: &[String]) {
        for window in tokens.windows(self.n + 1) {
            let key = window[..self.n].to_vec();
            let value = window[self.n].clone();
            self.model.entry(key).or_insert_with(Vec::new).push(value);
        }
        self.update_stats();
    }

    pub fn predict_next<R: rand::Rng>(&self, key: &[String], rng: &mut R) -> Option<String> {
        self.model
            .get(key)
            .and_then(|values| values.choose(rng).cloned())
    }

    pub fn update_stats(&mut self) {
        let mut stats = ModelStats::default();

        stats.unique_contexts = self.model.len();

        let mut total_tokens = 0;
        let mut unique_continuations = 0;
        let mut largest_set = 0;

        for values in self.model.values() {
            total_tokens += values.len();

            let mut unique = HashMap::new();
            for token in values.iter() {
                *unique.entry(token).or_insert(0) += 1;
            }

            unique_continuations += unique.len();
            largest_set = largest_set.max(values.len());
        }

        stats.total_tokens = total_tokens;
        stats.unique_continuations = unique_continuations;
        stats.largest_continuation_set = largest_set;

        self.stats = stats;
    }

    pub fn prune(&mut self, min_occurrences: usize) -> usize {
        let mut pruned_count = 0;

        let mut pruned_model = HashMap::new();

        for (context, continuations) in &self.model {
            let mut counts = HashMap::new();
            for token in continuations {
                *counts.entry(token).or_insert(0) += 1;
            }

            let mut new_continuations = Vec::new();
            for token in continuations {
                if counts[token] >= min_occurrences {
                    new_continuations.push(token.clone());
                } else {
                    pruned_count += 1;
                }
            }

            if !new_continuations.is_empty() {
                pruned_model.insert(context.clone(), new_continuations);
            }
        }

        self.model = pruned_model;
        self.update_stats();

        pruned_count
    }

    pub fn optimize(&mut self) -> usize {
        let mut optimized_count = 0;

        let mut optimized_model = HashMap::new();

        for (context, continuations) in &self.model {
            let mut counts = HashMap::new();
            for token in continuations {
                *counts.entry(token.clone()).or_insert(0) += 1;
            }

            let mut new_continuations = Vec::new();
            for (token, count) in counts {
                for _ in 0..count {
                    new_continuations.push(token.clone());
                }
            }

            optimized_count += continuations.len() - new_continuations.len();
            optimized_model.insert(context.clone(), new_continuations);
        }

        self.model = optimized_model;
        self.update_stats();

        optimized_count
    }

    pub fn get_stats(&self) -> &ModelStats {
        &self.stats
    }

    pub fn generate(&self, start_tokens: Option<Vec<String>>, max_tokens: usize) -> Vec<String> {
        let mut result = Vec::new();
        let mut current_tokens = start_tokens.unwrap_or_default();

        for _ in 0..max_tokens {
            if let Some(next_token) = self.next_token(&current_tokens) {
                result.push(next_token.clone());
                current_tokens.push(next_token);
                if current_tokens.len() > self.n {
                    current_tokens.remove(0);
                }
            } else {
                break;
            }
        }

        result
    }

    fn next_token(&self, context: &[String]) -> Option<String> {
        let mut rng = rng();
        self.predict_next(context, &mut rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rng;

    #[test]
    fn test_add_tokens() {
        let mut model = NGramModel::new(2);
        let tokens = vec![
            "the".to_string(),
            "quick".to_string(),
            "brown".to_string(),
            "fox".to_string(),
            "jumps".to_string(),
            "over".to_string(),
        ];

        model.add_tokens(&tokens);

        let context = vec!["the".to_string(), "quick".to_string()];
        let mut rng = rng();
        assert_eq!(
            model.predict_next(&context, &mut rng),
            Some("brown".to_string())
        );
    }

    #[test]
    fn test_prune() {
        let mut model = NGramModel::new(1);
        let tokens = vec![
            "a".to_string(),
            "b".to_string(),
            "a".to_string(),
            "b".to_string(),
            "a".to_string(),
            "c".to_string(),
        ];

        model.add_tokens(&tokens);

        let pruned = model.prune(2);

        assert_eq!(pruned, 1);

        let context = vec!["a".to_string()];
        let mut rng = rng();
        assert_eq!(
            model.predict_next(&context, &mut rng),
            Some("b".to_string())
        );
    }
}
