use anyhow::anyhow;
use anyhow::{Ok, Result};
use data::ProcessRecords;
use parser::LogcatParser;
use process_stream::{Process, ProcessExt, StreamExt};
use utils::Terminal;
mod data;
mod parser;
mod record;
mod utils;
#[tokio::main]
async fn main() -> Result<()> {
    let logcat_parser = LogcatParser {};
    let process_records = ProcessRecords {
        enabled: true,
        ..ProcessRecords::default()
    };
    let mut process: Process = vec!["adb", "logcat"].into();
    let process_records_clone = process_records.clone();
    let mut terminal = Terminal::default();
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
