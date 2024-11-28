use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

mod crypto;
mod git;
mod storage;
mod cert;

pub use crypto::Crypto;
pub use git::GitManager;
pub use storage::Storage;
use cert::CertManager;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Git error: {0}")]
    Git(String),
    #[error("JSON error: {0}")]
    Json(String),
    #[error("Certificate error: {0}")]
    Certificate(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub data: serde_json::Value,
    pub created_at: u64,
    pub updated_at: u64,
}

pub struct GithubDb {
    storage: Storage,
    git: GitManager,
    crypto: Option<Crypto>,
    cert_manager: CertManager,
}

impl GithubDb {
    pub fn new<P: AsRef<Path>>(path: P, encryption_key: Option<&[u8]>) -> Result<Self> {
        let storage = Storage::new(path.as_ref())?;
        let git = GitManager::new(path.as_ref())?;
        let crypto = if let Some(key) = encryption_key {
            Some(Crypto::new(key)?)
        } else {
            None
        };
        let cert_manager = CertManager::new(path.as_ref())?;

        Ok(Self {
            storage,
            git,
            crypto,
            cert_manager,
        })
    }

    pub fn generate_certificate(&self, username: &str) -> Result<(Vec<u8>, Vec<u8>)> {
        self.cert_manager.generate_cert(username)
    }

    pub fn verify_certificate(&self, cert_data: &[u8]) -> Result<bool> {
        // Extract username from certificate
        let cert = openssl::x509::X509::from_pem(cert_data)
            .map_err(|e| DbError::Certificate(format!("Invalid certificate: {}", e)))?;
        
        let subject_name = cert.subject_name();
        let cn = subject_name.entries_by_nid(openssl::nid::Nid::COMMONNAME)
            .next()
            .ok_or_else(|| DbError::Certificate("No username found in certificate".to_string()))?;
        
        let username = cn.data().as_utf8()
            .map_err(|e| DbError::Certificate(format!("Invalid username encoding: {}", e)))?;

        self.cert_manager.verify_cert(username.to_string().as_str(), cert_data)
    }

    pub fn revoke_certificate(&self, username: &str) -> Result<()> {
        self.cert_manager.revoke_cert(username)
    }

    pub fn list_certificates(&self) -> Result<Vec<String>> {
        self.cert_manager.list_certs()
    }

    pub fn create(&mut self, id: &str, data: serde_json::Value) -> Result<Document> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let doc = Document {
            id: id.to_string(),
            data,
            created_at: now,
            updated_at: now,
        };

        let json = serde_json::to_string(&doc)?;
        let encrypted = if let Some(crypto) = &self.crypto {
            crypto.encrypt(json.as_bytes())?
        } else {
            json.into_bytes()
        };

        self.storage.write(&doc.id, &encrypted)?;
        self.git.commit(&format!("Create document {}", doc.id))?;

        Ok(doc)
    }

    pub fn read(&self, id: &str) -> Result<Document> {
        let data = self.storage.read(id)?;
        let json = if let Some(crypto) = &self.crypto {
            String::from_utf8(crypto.decrypt(&data)?)?
        } else {
            String::from_utf8(data)?
        };

        Ok(serde_json::from_str(&json)?)
    }

    pub fn update(&mut self, id: &str, data: serde_json::Value) -> Result<Document> {
        let mut doc = self.read(id)?;
        doc.data = data;
        doc.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let json = serde_json::to_string(&doc)?;
        let encrypted = if let Some(crypto) = &self.crypto {
            crypto.encrypt(json.as_bytes())?
        } else {
            json.into_bytes()
        };

        self.storage.write(&doc.id, &encrypted)?;
        self.git.commit(&format!("Update document {}", doc.id))?;

        Ok(doc)
    }

    pub fn delete(&mut self, id: &str) -> Result<()> {
        self.storage.delete(id)?;
        self.git.commit(&format!("Delete document {}", id))?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<String>> {
        self.storage.list()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn test_crud_operations() -> Result<()> {
        let dir = tempdir()?;
        let mut db = GithubDb::new(dir.path(), None)?;

        // Generate test certificate
        let (cert, _) = db.generate_certificate("testuser")?;
        assert!(db.verify_certificate(&cert)?);

        // Test create
        let doc = db.create("test1", json!({ "name": "Test Document" }))?;
        assert_eq!(doc.id, "test1");
        assert_eq!(doc.data["name"], "Test Document");

        // Test read
        let read_doc = db.read("test1")?;
        assert_eq!(read_doc.id, doc.id);
        assert_eq!(read_doc.data, doc.data);

        // Test update
        let updated = db.update("test1", json!({ "name": "Updated Document" }))?;
        assert_eq!(updated.id, "test1");
        assert_eq!(updated.data["name"], "Updated Document");

        // Test list
        let docs = db.list()?;
        assert_eq!(docs, vec!["test1"]);

        // Test delete
        db.delete("test1")?;
        assert!(db.read("test1").is_err());

        Ok(())
    }

    #[test]
    fn test_encryption() -> Result<()> {
        let dir = tempdir()?;
        let key = [0u8; 32]; // 32-byte key filled with zeros for testing
        let mut db = GithubDb::new(dir.path(), Some(&key))?;

        // Generate test certificate
        let (cert, _) = db.generate_certificate("testuser")?;
        assert!(db.verify_certificate(&cert)?);

        let doc = db.create("test1", json!({ "secret": "Classified" }))?;
        let read_doc = db.read("test1")?;
        assert_eq!(read_doc.data, doc.data);

        Ok(())
    }

    #[test]
    fn test_certificate_management() -> Result<()> {
        let dir = tempdir()?;
        let db = GithubDb::new(dir.path(), None)?;

        // Generate certificate
        let username = "testuser";
        let (cert, _) = db.generate_certificate(username)?;

        // Verify certificate
        assert!(db.verify_certificate(&cert)?);

        // List certificates
        let certs = db.list_certificates()?;
        assert_eq!(certs, vec!["testuser"]);

        // Revoke certificate
        db.revoke_certificate(username)?;
        assert!(!db.verify_certificate(&cert)?);

        Ok(())
    }
}
