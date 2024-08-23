use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Result, Write},
};

use tracing::warn;

use super::Record;

pub struct Recorder {
    record_file_path: String,
    record: Record,
}

impl Recorder {
    pub fn new(record_file_dir: String) -> Self {
        let record_file_path = format!("{}/record.json", record_file_dir);
        // 如果文件不存在，创建一个新的，并写入默认数据
        // 如果文件存在，读取文件内容
        let record: Record;

        let mut record_file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&record_file_path)
        {
            Ok(f) => f,
            Err(_) => {
                panic!("Failed to open record file: {}", record_file_path);
            }
        };

        {
            let result = Self::read_from_file(&record_file);
            record = match result {
                Ok(r) => r,
                Err(_) => {
                    // 文件内容无法解析，重置为默认值
                    warn!("Record data is invalid, reset to default.");
                    let record = Record::new();
                    Self::write_to_file(&mut record_file, &record).unwrap();
                    record
                }
            };
        }

        Recorder {
            record_file_path,
            record,
        }
    }

    fn read_from_file(file: &File) -> Result<Record> {
        // Check if file exist
        let record: Record;

        match serde_json::from_reader(BufReader::new(file)) {
            Ok(r) => record = r,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse record file: {}", e),
                ));
            }
        }

        Ok(record)
    }

    fn write_to_file(file: &mut File, record: &Record) -> Result<()> {
        file.write_all(serde_json::to_string(record)?.as_bytes())
    }

    pub fn get_record(&self) -> Record {
        self.record.clone()
    }

    pub fn update_record(&mut self, record: Record) {
        self.record = record;
        let mut record_file = match OpenOptions::new()
            .write(true)
            .create(true)
            .append(false)
            .open(&self.record_file_path)
        {
            Ok(f) => f,
            Err(_) => {
                panic!("Failed to open record file: {}", &self.record_file_path);
            }
        };
        Self::write_to_file(&mut record_file, &self.record).unwrap();
    }
}
