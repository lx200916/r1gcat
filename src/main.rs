use anyhow::anyhow;
use anyhow::{Ok, Result};
use clap::Parser;
use data::ProcessRecords;
use parser::LogcatParser;
use process_stream::{Process, ProcessExt, StreamExt};
use utils::Terminal;

mod data;
mod parser;
mod record;
mod utils;
#[derive(Parser, Debug)]
#[clap(name = "logcat")]
struct Args {
    #[clap(long)]
    pub hide_timestamp: bool,
    #[clap(long, default_value_t = true)]
    pub hide_date: bool,
    #[clap(long, short = 'p', default_value_t = true)]
    pub use_process_name: bool,
    #[clap(long)]
    pub bright_colors: bool,
    #[clap(long, short)]
    pub filter: Vec<String>,
    #[clap(long)]
    pub process_name_width: Option<usize>,
    #[clap(long)]
    pub tag_width: Option<usize>,
    #[clap(long)]
    pub pid_width: Option<usize>,
}
impl From<Args> for Terminal {
    fn from(args: Args) -> Self {
        let mut terminal = Terminal::default();
        terminal.hide_timestamp = args.hide_timestamp;
        terminal.hide_date = args.hide_date;
        terminal.use_process_name = args.use_process_name;
        terminal.bright_colors = args.bright_colors;
        if let Some(width) = args.process_name_width {
            terminal.process_name_width = width;
        }
        if let Some(width) = args.tag_width {
            terminal.tag_width = width;
        }
        if let Some(width) = args.pid_width {
            terminal.pid_width = width;
        }
        terminal
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();
    let adb_path = utils::adb()?;
    let process_records = ProcessRecords {
        enabled: args.use_process_name,
        adb_cmd: adb_path.to_str().unwrap_or("adb").to_string(),
        ..ProcessRecords::default()
    };
    let mut terminal: Terminal = args.into();
    let logcat_parser = LogcatParser {};
    let mut process: Process = vec![adb_path.to_str().unwrap_or("adb"), "logcat"].into();
    let process_records_clone = process_records.clone();
    tokio::spawn(async move {
        process_records_clone.update_process_record().await;
    });
    let mut stream = process.spawn_and_stream()?;
    while let Some(line) = stream.next().await {
        match line {
            process_stream::ProcessItem::Output(line) => {
                let record = logcat_parser.try_parse(&line);
                if let Some(mut record) = record {
                    record.process_name = process_records.get_process_name(record.pid);
                    // println!("{:?}",record.process_name)
                    terminal.print(&record)?;
                }
            }
            process_stream::ProcessItem::Error(err) => {
                return Err(anyhow!(err));
            }
            process_stream::ProcessItem::Exit(code) => {
                return Err(anyhow!("exit code:{:?}", code));
            }
        }
    }
    Ok(())
}
