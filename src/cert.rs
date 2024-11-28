use anyhow::Result;
use openssl::{
    pkey::{PKey, Private, Public},
    rsa::Rsa,
    x509::{X509Builder, X509},
};
use std::{fs, path::{Path, PathBuf}};
use crate::DbError;

pub struct CertManager {
    certs_path: PathBuf,
}

impl CertManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let certs_path = path.as_ref().join("certs");
        fs::create_dir_all(&certs_path)?;
        Ok(Self { certs_path })
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

        // Save certificate and private key
        let cert_path = self.certs_path.join(format!("{}.cert", username));
        let key_path = self.certs_path.join(format!("{}.key", username));
        
        fs::write(&cert_path, certificate.to_pem()?)?;
        fs::write(&key_path, private_key.private_key_to_pem_pkcs8()?)?;

        Ok((certificate.to_pem()?, private_key.private_key_to_pem_pkcs8()?))
    }

    pub fn verify_cert(&self, username: &str, cert_data: &[u8]) -> Result<bool> {
        let cert = X509::from_pem(cert_data)
            .map_err(|e| DbError::Storage(format!("Invalid certificate: {}", e)))?;

        // Check if certificate exists in our store
        let stored_cert_path = self.certs_path.join(format!("{}.cert", username));
        if !stored_cert_path.exists() {
            return Ok(false);
        }

        let stored_cert_data = fs::read(&stored_cert_path)?;
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
        let cert_manager = CertManager::new(dir.path())?;

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
}
