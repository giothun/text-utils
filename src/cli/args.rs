use clap::Parser;
use std::path::PathBuf;
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CliArgs {
    /// Scraper type to use for fetching text
    ///
    /// Available options:
    ///   - "basic": Scrapes text from a website using a CSS selector
    ///   - "wikipedia": Fetches summaries from Wikipedia for specified topics
    ///   - "lyrics": Fetches song lyrics from Genius for specified artists
    #[arg(short = 't', long, default_value = "basic")]
    pub scraper_type: String,

    /// Path to a scraper configuration JSON file
    ///
    /// If provided, the scraper will be configured using this file instead
    /// of prompting for interactive configuration.
    #[arg(short = 'c', long)]
    pub scraper_config: Option<PathBuf>,

    /// N-gram size to use for the model
    ///
    /// Higher values create more coherent but less creative text.
    /// Recommended values: 2-4
    #[arg(short = 'n', long = "ngram", default_value_t = 3)]
    pub n: usize,

    /// Number of tokens to generate
    ///
    /// This determines the length of the generated text output.
    #[arg(short, long, default_value_t = 50)]
    pub length: usize,

    /// Path to save the generated text
    ///
    /// If not provided, text will be printed to the console.
    #[arg(short, long)]
    pub output_file: Option<PathBuf>,

    /// Seed text to start generation
    ///
    /// If provided, generation will start with these words.
    /// Example: --seed "Once upon a time"
    #[arg(long)]
    pub seed: Option<String>,

    /// Path to a local text file to use as training data
    ///
    /// If provided, this file will be used instead of scraping text.
    #[arg(short = 'i', long)]
    pub input_file: Option<PathBuf>,

    /// Whether to convert text to lowercase during tokenization
    ///
    /// Lowercase conversion reduces vocabulary size and improves pattern recognition.
    #[arg(long, default_value_t = true)]
    pub lowercase: bool,

    /// Whether to preserve punctuation as separate tokens
    ///
    /// Preserving punctuation helps maintain sentence structure and readability.
    #[arg(long, default_value_t = true)]
    pub preserve_punctuation: bool,

    /// Whether to preserve sentence boundaries during tokenization
    ///
    /// This prevents the model from generating nonsensical transitions between sentences.
    #[arg(long, default_value_t = true)]
    pub preserve_sentence_boundaries: bool,

    /// Minimum occurrences for pruning rare n-grams
    ///
    /// Higher values create smaller models but may reduce quality.
    /// 0 means no pruning.
    #[arg(long, default_value_t = 0)]
    pub prune_min_occurrences: usize,

    /// Optimize the model by deduplicating continuations
    ///
    /// This reduces model size without affecting generation quality.
    #[arg(long)]
    pub optimize: bool,

    /// Show detailed model statistics after training
    ///
    /// Displays information about the model size, token counts, and other metrics.
    #[arg(long)]
    pub show_stats: bool,

    /// Disable wordcloud generation
    ///
    /// By default, a wordcloud image is generated from the output text.
    /// Use this flag to disable this feature.
    #[arg(long)]
    pub no_wordcloud: bool,

    /// Disable text insights and statistics
    ///
    /// By default, insights and statistics about the generated text are displayed.
    /// Use this flag to disable this feature.
    #[arg(long)]
    pub no_insights: bool,

    /// Enable verbose logging
    ///
    /// This will print additional information during execution.
    #[arg(short, long)]
    pub verbose: bool,
}

pub fn parse_args() -> CliArgs {
    CliArgs::parse()
}
