use ::futures::future::try_join_all;
use cloud_storage::{Client, ListRequest};
use futures::TryStreamExt;
use speedometer::log::LogRecord;
use std::{fs::File, path::PathBuf};

use indicatif::{ProgressBar, ProgressStyle};

use clap::Parser;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if let Some(service_account_file) = cli.auth_file {
        env::set_var("SERVICE_ACCOUNT", service_account_file)
    }
    let client = Client::new();
    let objects = client
        .object()
        .list(&cli.bucket, ListRequest::default())
        .await?
        .map_ok(|object_list| object_list.items)
        .try_concat()
        .await?;
    let total_size = objects.iter().map(|o| o.size).sum::<u64>();
    let progress_bar = ProgressBar::new(total_size)
        .with_style(
            ProgressStyle::with_template(
                "[{elapsed}] {bar:40.white.on_white.dim} {percent}% {msg}",
            )
            .unwrap(),
        )
        .with_message("Downloading speedtest logs");
    let download_tasks = objects.into_iter().map(|object| async {
        let object = object;
        let bytes = Client::new()
            .object()
            .download(&object.bucket, &object.name)
            .await;
        progress_bar.inc(object.size);
        bytes
    });
    let log_bytes = try_join_all(download_tasks).await?;
    progress_bar.finish();
    let log_files = log_bytes
        .into_iter()
        .flat_map(|bytes| LogRecord::from_json(&bytes))
        .collect::<Vec<_>>();

    let output_filename = cli.output.unwrap_or_else(|| PathBuf::from("output.csv"));
    let f = File::create(output_filename)?;
    let mut csv_w = csv::Writer::from_writer(f);

    csv_w.write_record(LogRecord::HEADERS)?;
    for log_file in log_files {
        csv_w.serialize(log_file)?;
    }
    csv_w.flush()?;

    Ok(())
}

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[arg(short = 'f', long = "auth-file", value_name = "SERVICE-ACCOUNT.JSON")]
    auth_file: Option<PathBuf>,
    #[arg(
        short,
        long,
        value_name = "BUCKET",
        default_value_t = String::from("better-broadband-monitoring-logs")
    )]
    bucket: String,
    #[arg(short, long, value_name = "OUTPUT.CSV")]
    output: Option<PathBuf>,
}
