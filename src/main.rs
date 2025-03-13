use text_gen_ngram::{app::run_app, cli, error::Result as TextGenResult};

use log::info;
use tokio::signal;

#[tokio::main]
async fn main() -> TextGenResult<()> {
    let args = cli::parse_args();

    env_logger::Builder::new()
        .filter_level(if args.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();

    info!("Starting text-gen-ngram");

    let ctrl_c = signal::ctrl_c();

    tokio::select! {
        result = run_app(args) => result,
        _ = ctrl_c => {
            info!("Application terminated by user");
            Ok(())
        }
    }
}
