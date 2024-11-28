# GitHub DB

A secure, Git LFS-backed JSON database with certificate-based authentication and native GitHub Actions support.

## Quick Start

### For Maintainers

1. Setup repositories:
```bash
# Clone this repository
git clone https://github.com/your-username/github-db
cd github-db

# Run setup script
chmod +x setup.sh
./setup.sh

# Create first release
git tag -a v0.1.0 -m "Initial release"
git push origin v0.1.0
```

### For Users

1. Create database from template:
```bash
gh repo create my-database --template OWNER/github-db-template
cd my-database
```

2. Set up authentication:
```bash
# Download latest binary
curl -L -o github-db \
  https://github.com/OWNER/github-db/releases/latest/download/github-db-linux-x86_64
chmod +x github-db

# Generate certificate
./github-db generate-cert my-cert -o ./certs
gh secret set DB_CERT -b"$(cat certs/my-cert.cert | base64)"

# Optional: Enable encryption
gh secret set DB_KEY -b"$(openssl rand -base64 32)"
```

3. Use the database:
```bash
# Add documents through files
echo '{"name": "test"}' > data/doc1.json
git add data/doc1.json
git commit -m "Add doc1"
git push

# Or use GitHub Actions
gh workflow run database.yml -f operation=create -f id=doc2 -f data='{"name": "test2"}'
```

## Features

- **Zero Dependencies**: Single static binary
- **Git LFS Storage**: Efficient handling of large datasets
- **Certificate Authentication**: Secure access control
- **Optional Encryption**: AES-256-GCM for sensitive data
- **GitHub Actions Ready**: Native CI/CD integration

## Security

- Certificate-based authentication
- Optional AES-256-GCM encryption
- Git LFS for efficient storage
- Automated maintenance
- Full audit trail

## Documentation

- [Setup Instructions](SETUP.md)
- [Template README](template/README.md)
- [Example Workflow](.github/workflows/example.yml)

## Updating

When a new version is released:

1. Update workflow file:
```yaml
# In .github/workflows/database.yml
curl -L -o github-db https://github.com/OWNER/github-db/releases/download/vX.Y.Z/github-db-linux-x86_64
```

2. Commit and push:
```bash
git commit -am "Update to vX.Y.Z"
git push
```

## License

MIT License - See [LICENSE](LICENSE) for details
