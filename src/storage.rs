use anyhow::Result;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let base_path = path.as_ref().to_path_buf();
        fs::create_dir_all(&base_path)?;
        
        Ok(Self { base_path })
    }

    fn get_file_path(&self, id: &str) -> PathBuf {
        self.base_path.join(format!("{}.json", id))
    }

    pub fn write(&self, id: &str, data: &[u8]) -> Result<()> {
        let path = self.get_file_path(id);
        let mut file = File::create(path)?;
        file.write_all(data)?;
        Ok(())
    }

    pub fn read(&self, id: &str) -> Result<Vec<u8>> {
        let path = self.get_file_path(id);
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(data)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let path = self.get_file_path(id);
        fs::remove_file(path)?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    files.push(file_name[..file_name.len() - 5].to_string());
                }
            }
        }
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_storage_operations() {
        let dir = tempdir().unwrap();
        let storage = Storage::new(dir.path()).unwrap();

        // Test write and read
        let id = "test";
        let data = b"Hello, World!";
        storage.write(id, data).unwrap();
        let read_data = storage.read(id).unwrap();
        assert_eq!(data.to_vec(), read_data);

        // Test list
        let files = storage.list().unwrap();
        assert_eq!(files, vec!["test"]);

        // Test delete
        storage.delete(id).unwrap();
        assert!(storage.read(id).is_err());
    }
}
