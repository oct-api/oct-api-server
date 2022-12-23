use std::{fs, path::Path};
use std::io::prelude::*;
use std::process::Command;
use serde_json;
use crate::types::*;

#[derive(Debug)]
pub struct Entry {
    name: String,
}

impl Entry {
    pub fn new(name: &str) -> Entry {
        Entry {
            name: name.to_string(),
        }
    }

    pub fn child(&self, name: &str) -> Entry {
        Entry {
            name: format!("{}/{}", self.name, name),
        }
    }

    pub fn fullpath(&self) -> String {
        format!("{}/{}", config().data_dir, self.name)
    }

    pub async fn size(&self) -> Result<u64> {
        let out = Command::new("du")
            .args(&["-sb", &self.fullpath()])
            .output()?;
        let line = String::from_utf8(out.stdout)?;
        let fs: Vec<&str> = line.split('\t').collect();
        let r: u64 = fs[0].parse()?;
        Ok(r)
    }

    pub fn create_dirs(&self) -> Result<()> {
        fs::create_dir_all(self.fullpath())?;
        Ok(())
    }

    pub fn exists(&self) -> bool {
        Path::new(&self.fullpath()).exists()
    }

    pub async fn read(&self) -> Result<String> {
        Ok(fs::read_to_string(self.fullpath())?)
    }

    pub async fn write<T: AsRef<[u8]>>(&self, data: T) -> Result<()> {
        if let Some(d) = Path::new(&self.fullpath()).parent() {
            fs::create_dir_all(&d)?;
        }
        Ok(fs::write(self.fullpath(), data)?)
    }

    pub async fn append<T: AsRef<[u8]>>(&self, data: T) -> Result<()> {
        let fp = self.fullpath();
        if let Some(d) = Path::new(&fp).parent() {
            fs::create_dir_all(&d)?;
        }
        let mut f = fs::OpenOptions::new().create(true).append(true).open(fp)?;
        f.write_all(data.as_ref())?;
        Ok(())
    }

    pub async fn write_json<T: Serialize>(&self, data: &T) -> Result<()> {
        self.write(serde_json::to_string(data)?).await
    }

    pub fn listdir(&self) -> Result<Vec<String>> {
        let mut ret = Vec::new();
        let p = self.fullpath();
        let dir = Path::new(&p);
        if ! dir.exists() {
            return Ok(ret);
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            ret.push(entry.file_name().to_str().unwrap().to_string());
        }
        Ok(ret)
    }
}
