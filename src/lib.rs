use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use std::cmp::Ordering;

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
    #[error("Filter error: {0}")]
    Filter(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub data: serde_json::Value,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOp {
    Eq,
    Gt,
    Lt,
    Gte,
    Lte,
    Contains,
    StartsWith,
    EndsWith,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCondition {
    pub field: String,
    pub op: FilterOp,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Filter {
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Condition(FilterCondition),
}

fn compare_values(a: &serde_json::Value, b: &serde_json::Value) -> Result<Ordering> {
    match (a, b) {
        (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
            if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                Ok(a.partial_cmp(&b).unwrap_or(Ordering::Equal))
            } else {
                Err(anyhow::anyhow!("Invalid number comparison"))
            }
        },
        (serde_json::Value::String(a), serde_json::Value::String(b)) => {
            Ok(a.cmp(b))
        },
        (serde_json::Value::Bool(a), serde_json::Value::Bool(b)) => {
            Ok(a.cmp(b))
        },
        _ => Err(anyhow::anyhow!("Cannot compare values of different types")),
    }
}

impl Filter {
    fn matches(&self, doc: &Document) -> Result<bool> {
        match self {
            Filter::And(filters) => {
                for filter in filters {
                    if !filter.matches(doc)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            },
            Filter::Or(filters) => {
                for filter in filters {
                    if filter.matches(doc)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            },
            Filter::Condition(condition) => {
                let value = get_nested_value(&doc.data, &condition.field)
                    .ok_or_else(|| anyhow::anyhow!("Field not found: {}", condition.field))?;

                match &condition.op {
                    FilterOp::Eq => Ok(value == &condition.value),
                    FilterOp::Gt => Ok(compare_values(value, &condition.value)? == Ordering::Greater),
                    FilterOp::Lt => Ok(compare_values(value, &condition.value)? == Ordering::Less),
                    FilterOp::Gte => {
                        let cmp = compare_values(value, &condition.value)?;
                        Ok(cmp == Ordering::Greater || cmp == Ordering::Equal)
                    },
                    FilterOp::Lte => {
                        let cmp = compare_values(value, &condition.value)?;
                        Ok(cmp == Ordering::Less || cmp == Ordering::Equal)
                    },
                    FilterOp::Contains => {
                        match (value, &condition.value) {
                            (serde_json::Value::String(field), serde_json::Value::String(pattern)) => {
                                Ok(field.contains(pattern))
                            },
                            _ => Err(anyhow::anyhow!("Contains operation requires string values")),
                        }
                    },
                    FilterOp::StartsWith => {
                        match (value, &condition.value) {
                            (serde_json::Value::String(field), serde_json::Value::String(pattern)) => {
                                Ok(field.starts_with(pattern))
                            },
                            _ => Err(anyhow::anyhow!("StartsWith operation requires string values")),
                        }
                    },
                    FilterOp::EndsWith => {
                        match (value, &condition.value) {
                            (serde_json::Value::String(field), serde_json::Value::String(pattern)) => {
                                Ok(field.ends_with(pattern))
                            },
                            _ => Err(anyhow::anyhow!("EndsWith operation requires string values")),
                        }
                    },
                }
            }
        }
    }
}

fn get_nested_value<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;
    
    for part in parts {
        current = current.get(part)?;
    }
    
    Some(current)
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
        let cert_manager = CertManager::new(path.as_ref(), encryption_key)?;

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

    pub fn find(&self, filter: Option<Filter>) -> Result<Vec<Document>> {
        let ids = self.list()?;
        let mut results = Vec::new();

        for id in ids {
            let doc = self.read(&id)?;
            if let Some(filter) = &filter {
                if filter.matches(&doc)? {
                    results.push(doc);
                }
            } else {
                results.push(doc);
            }
        }

        Ok(results)
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
        let key = [0u8; 32];
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

    #[test]
    fn test_filters() -> Result<()> {
        let dir = tempdir()?;
        let mut db = GithubDb::new(dir.path(), None)?;

        // Generate test certificate
        let (cert, _) = db.generate_certificate("testuser")?;
        assert!(db.verify_certificate(&cert)?);

        // Create test documents
        db.create("user1", json!({
            "name": "Alice",
            "age": 25,
            "city": "New York"
        }))?;

        db.create("user2", json!({
            "name": "Bob",
            "age": 30,
            "city": "San Francisco"
        }))?;

        // Test equality filter
        let filter = Filter::Condition(FilterCondition {
            field: "name".to_string(),
            op: FilterOp::Eq,
            value: json!("Alice"),
        });
        let results = db.find(Some(filter))?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].data["name"], "Alice");

        // Test numeric comparison
        let filter = Filter::Condition(FilterCondition {
            field: "age".to_string(),
            op: FilterOp::Gt,
            value: json!(27),
        });
        let results = db.find(Some(filter))?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].data["name"], "Bob");

        // Test AND filter
        let filter = Filter::And(vec![
            Filter::Condition(FilterCondition {
                field: "age".to_string(),
                op: FilterOp::Gte,
                value: json!(25),
            }),
            Filter::Condition(FilterCondition {
                field: "city".to_string(),
                op: FilterOp::Contains,
                value: json!("York"),
            }),
        ]);
        let results = db.find(Some(filter))?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].data["name"], "Alice");

        Ok(())
    }
}
