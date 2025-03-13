use crate::{
    cli::CliArgs,
    error::{Result as TextGenResult, TextGenError},
    model::{Generator, NGramModel},
    output::insights::show_text_insights,
    output::wordcloud::generate_wordcloud,
};

use log::info;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::training::train_new_model;

pub async fn run_app(args: CliArgs) -> TextGenResult<()> {
    let mut model = train_new_model(&args).await?;

    if args.optimize {
        info!("Optimizing model...");
        let optimized_count = model.optimize();
        info!("Optimized {} token occurrences", optimized_count);
    }

    if args.prune_min_occurrences > 0 {
        info!(
            "Pruning model (min occurrences: {})...",
            args.prune_min_occurrences
        );
        let pruned_count = model.prune(args.prune_min_occurrences);
        info!("Pruned {} token occurrences", pruned_count);
    }

    if args.show_stats {
        display_model_stats(&model);
    }

    let generated_text = generate_text(&model, &args);

    if let Some(output_file) = &args.output_file {
        info!("Saving generated text to {}", output_file.display());
        let mut file = fs::File::create(output_file)
            .await
            .map_err(|e| TextGenError::Io(e))?;
        file.write_all(generated_text.as_bytes())
            .await
            .map_err(|e| TextGenError::Io(e))?;
        info!("Generated text saved successfully");
    } else {
        println!("\nGenerated text:\n{}", generated_text);
    }

    if !args.no_insights {
        show_text_insights(&generated_text);
    }

    if !args.no_wordcloud {
        info!("Generating wordcloud from the text...");
        let wordcloud_path = generate_wordcloud(&generated_text)?;
        info!("Wordcloud saved to {}", wordcloud_path);
    }

    info!("Text generation completed successfully");
    Ok(())
}

fn display_model_stats(model: &NGramModel) {
    let stats = model.get_stats();
    println!("\nModel Statistics:");
    println!("----------------");
    println!("N-gram size: {}", model.n);
    println!("Total token occurrences: {}", stats.total_tokens);
    println!("Unique contexts: {}", stats.unique_contexts);
    println!("Unique continuations: {}", stats.unique_continuations);
    println!(
        "Largest continuation set: {}",
        stats.largest_continuation_set
    );
    println!(
        "Average continuations per context: {:.2}",
        stats.total_tokens as f64 / stats.unique_contexts as f64
    );
}

fn generate_text(model: &NGramModel, args: &CliArgs) -> String {
    info!("Generating text (length: {} tokens)", args.length);
    let generator = Generator::new(model);
    let seed_words = args
        .seed
        .as_ref()
        .map(|s| s.split_whitespace().map(String::from).collect());
    generator.generate(seed_words, args.length)
}
