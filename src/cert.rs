use anyhow::Result;
use openssl::{
    pkey::{PKey, Private, Public},
    rsa::Rsa,
    x509::{X509Builder, X509},
};
use std::{fs, path::{Path, PathBuf}};
use crate::{DbError, Crypto};

pub struct CertManager {
    certs_path: PathBuf,
    crypto: Option<Crypto>,
}

impl CertManager {
    pub fn new<P: AsRef<Path>>(path: P, encryption_key: Option<&[u8]>) -> Result<Self> {
        let certs_path = path.as_ref().join("certs");
        fs::create_dir_all(&certs_path)?;
        
        let crypto = if let Some(key) = encryption_key {
            Some(Crypto::new(key)?)
        } else {
            None
        };

        Ok(Self { 
            certs_path,
            crypto,
        })
    }

    fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        if let Some(crypto) = &self.crypto {
            crypto.encrypt(data)
        } else {
            Ok(data.to_vec())
        }
    }

    fn decrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        if let Some(crypto) = &self.crypto {
            crypto.decrypt(data)
        } else {
            Ok(data.to_vec())
        }
    }

    pub fn generate_cert(&self, username: &str) -> Result<(Vec<u8>, Vec<u8>)> {
        // Generate RSA key pair
        let rsa = Rsa::generate(2048)?;
        let private_key = PKey::from_rsa(rsa)?;

        // Create certificate
        let mut builder = X509Builder::new()?;
        builder.set_version(2)?;
        let mut name_builder = openssl::x509::X509NameBuilder::new()?;
        name_builder.append_entry_by_text("CN", username)?;
        let name = name_builder.build();
        builder.set_subject_name(&name)?;
        builder.set_issuer_name(&name)?;
        builder.set_pubkey(&private_key)?;

        // Set validity period (1 year)
        let not_before = openssl::asn1::Asn1Time::days_from_now(0)?;
        let not_after = openssl::asn1::Asn1Time::days_from_now(365)?;
        builder.set_not_before(&not_before)?;
        builder.set_not_after(&not_after)?;

        // Sign the certificate
        builder.sign(&private_key, openssl::hash::MessageDigest::sha256())?;
        let certificate = builder.build();

        // Get PEM encoded data
        let cert_pem = certificate.to_pem()?;
        let key_pem = private_key.private_key_to_pem_pkcs8()?;

        // Encrypt and save certificate and private key
        let encrypted_cert = self.encrypt_data(&cert_pem)?;
        let encrypted_key = self.encrypt_data(&key_pem)?;

        let cert_path = self.certs_path.join(format!("{}.cert", username));
        let key_path = self.certs_path.join(format!("{}.key", username));
        
        fs::write(&cert_path, &encrypted_cert)?;
        fs::write(&key_path, &encrypted_key)?;

        Ok((cert_pem, key_pem))
    }

    pub fn verify_cert(&self, username: &str, cert_data: &[u8]) -> Result<bool> {
        let cert = X509::from_pem(cert_data)
            .map_err(|e| DbError::Storage(format!("Invalid certificate: {}", e)))?;

        // Check if certificate exists in our store
        let stored_cert_path = self.certs_path.join(format!("{}.cert", username));
        if !stored_cert_path.exists() {
            return Ok(false);
        }

        // Read and decrypt stored certificate
        let encrypted_cert_data = fs::read(&stored_cert_path)?;
        let stored_cert_data = self.decrypt_data(&encrypted_cert_data)?;
        let stored_cert = X509::from_pem(&stored_cert_data)?;

        // Compare certificates
        Ok(cert.to_pem()? == stored_cert.to_pem()?)
    }

    pub fn revoke_cert(&self, username: &str) -> Result<()> {
        let cert_path = self.certs_path.join(format!("{}.cert", username));
        let key_path = self.certs_path.join(format!("{}.key", username));
        
        if cert_path.exists() {
            fs::remove_file(cert_path)?;
        }
        if key_path.exists() {
            fs::remove_file(key_path)?;
        }
        
        Ok(())
    }

    pub fn list_certs(&self) -> Result<Vec<String>> {
        let mut certs = Vec::new();
        for entry in fs::read_dir(&self.certs_path)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if file_name.ends_with(".cert") {
                if let Some(username) = file_name.strip_suffix(".cert") {
                    certs.push(username.to_string());
                }
            }
        }
        Ok(certs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_certificate_operations() -> Result<()> {
        let dir = tempdir()?;
        let cert_manager = CertManager::new(dir.path(), None)?;

        // Generate certificate
        let username = "testuser";
        let (cert, _key) = cert_manager.generate_cert(username)?;

        // Verify certificate
        assert!(cert_manager.verify_cert(username, &cert)?);

        // List certificates
        let certs = cert_manager.list_certs()?;
        assert_eq!(certs, vec!["testuser"]);

        // Revoke certificate
        cert_manager.revoke_cert(username)?;
        assert!(!cert_manager.verify_cert(username, &cert)?);

        Ok(())
    }

    #[test]
    fn test_encrypted_certificates() -> Result<()> {
        let dir = tempdir()?;
        let key = [0u8; 32]; // 32-byte key for testing
        let cert_manager = CertManager::new(dir.path(), Some(&key))?;

        // Generate and verify encrypted certificate
        let username = "testuser";
        let (cert, _key) = cert_manager.generate_cert(username)?;
        assert!(cert_manager.verify_cert(username, &cert)?);

        // Verify the stored file is actually encrypted
        let cert_path = dir.path().join("certs").join("testuser.cert");
        let stored_data = fs::read(cert_path)?;
        assert_ne!(&stored_data, &cert); // Stored data should be encrypted

        Ok(())
    }
}
