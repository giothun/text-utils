pub mod insights;
pub mod wordcloud;

// Re-export main functions for convenience
pub use insights::show_text_insights;
pub use wordcloud::generate_wordcloud;
