use crate::parser::PSParser;
use crate::record::ProcessRecord;
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, RwLock};
#[derive(Clone, Debug)]
pub struct ProcessRecords {
    pub records: Arc<RwLock<HashMap<u32, ProcessRecord>>>,
    pub enabled: bool,
    pub adb_cmd: String,
}
impl Default for ProcessRecords {
    fn default() -> Self {
        ProcessRecords {
            records: Arc::new(RwLock::new(HashMap::new())),
            enabled: false,
            adb_cmd: "adb".to_string(),
        }
    }
}

impl ProcessRecords {
    pub(crate) fn get_process_record_map(&self, pid: u32) -> Option<ProcessRecord> {
        let records = self.records.read().unwrap();
        records.get(&pid).cloned()
    }
    pub fn get_process_name(&self, pid: u32) -> String {
        if !self.enabled {
            return format!("pid-{}", pid);
        }
        let mut process_name: Option<String> = self.get_process_record_map(pid).map(|r| r.name);
        if process_name.is_none() {
            let cmd = Command::new(&self.adb_cmd)
                .arg("shell")
                .arg("cat")
                .arg(format!("/proc/{}/cmdline", pid))
                .output();
            let cmdline = match cmd {
                Ok(cmd) => String::from_utf8_lossy(&cmd.stdout).to_string(),
                Err(_) => format!("pid-{}", pid),
            };
            self.records.write().unwrap().insert(
                pid,
                ProcessRecord {
                    name: cmdline.clone(),
                    pid,
                    ..ProcessRecord::default()
                },
            );
            process_name = Some(cmdline);
        };
        process_name.unwrap_or(format!("pid-{}", pid))
    }
    pub async fn update_process_record(&self) {
        let parser = PSParser {};
        #[allow(clippy::while_immutable_condition)]
        while self.enabled {
            let cmd = Command::new(&self.adb_cmd)
                .arg("shell")
                .arg("ps")
                .output()
                .expect("failed to execute process");
            let stdout = String::from_utf8_lossy(&cmd.stdout);
            {
                let mut records = self.records.write().unwrap();
                records.clear();
                stdout
                    .split("\n")
                    .collect::<Vec<&str>>()
                    .iter()
                    .skip(1)
                    .for_each(|line| {
                        let record = parser.try_parse(line);
                        if let Some(record) = record {
                            records.insert(record.pid, record);
                        }
                    });
            }
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    }
}
