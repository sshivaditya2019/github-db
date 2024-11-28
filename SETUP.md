# Setup Instructions

1. Create new database from template:
```bash
# Create repository from template
gh repo create my-database --template OWNER/github-db-template --public
cd my-database
```

2. Generate certificate and encryption key:
```bash
# Download latest binary
curl -L -o github-db \
  https://github.com/sshivaditya2019/github-db/releases/latest/download/github-db-linux-x86_64
chmod +x github-db

# Generate encryption key (recommended)
ENCRYPTION_KEY=$(openssl rand -hex 32)

# Generate encrypted certificate
./github-db --key "$ENCRYPTION_KEY" generate-cert my-cert -o ./certs
```

3. Set up repository secrets:
```bash
# Add certificate (required)
gh secret set DB_CERT -b"$(cat certs/my-cert.cert | base64)"

# Add encryption key (recommended for security)
gh secret set DB_KEY -b"$ENCRYPTION_KEY"
```

4. Start using the database:
```bash
# Add documents through data/ directory
echo '{"name": "test"}' > data/doc1.json
git add data/doc1.json
git commit -m "Add doc1"
git push

# Or use CLI with encryption
./github-db --key "$ENCRYPTION_KEY" --cert ./certs/my-cert.cert create doc1 '{"name": "test"}'
```

## Security Features

1. Certificate Encryption:
- Certificates are encrypted using AES-256-GCM
- Both certificate and private key files are protected
- Encryption key should be kept secure and never committed to the repository

2. Access Control:
- Each user needs their own certificate
- Certificates can be revoked if compromised
- All operations require a valid certificate

3. Data Protection:
- All stored documents are encrypted when DB_KEY is set
- Encryption uses AES-256-GCM with unique nonces
- Encrypted data is automatically handled by the CLI

## Managing Certificates

```bash
# Generate additional certificates
./github-db --key "$ENCRYPTION_KEY" generate-cert user2 -o ./certs

# List valid certificates
./github-db --key "$ENCRYPTION_KEY" --cert ./certs/my-cert.cert list-certs

# Revoke a certificate
./github-db --key "$ENCRYPTION_KEY" --cert ./certs/my-cert.cert revoke-cert user2
```

## Updating to New Versions

When a new version of github-db is released, update your database workflow:

1. Edit `.github/workflows/database.yml`:
```yaml
# Update the binary download URL
curl -L -o github-db https://github.com/OWNER/github-db/releases/download/vX.Y.Z/github-db-linux-x86_64
```

2. Commit and push:
```bash
git add .github/workflows/database.yml
git commit -m "Update to github-db vX.Y.Z"
git push
```

## Best Practices

1. Always use encryption in production:
   - Generate a strong encryption key
   - Store the key securely
   - Never commit encryption keys or certificates

2. Certificate Management:
   - Generate separate certificates for each user
   - Store encrypted certificates securely
   - Revoke certificates when no longer needed
   - Regularly rotate certificates (recommended yearly)

3. Access Control:
   - Limit certificate access to authorized users
   - Use GitHub repository secrets for CI/CD
   - Monitor certificate usage through Git history
