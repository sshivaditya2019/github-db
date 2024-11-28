# GitHub DB

A secure, Git LFS-backed JSON database with certificate-based authentication and native GitHub Actions support.

## Features

- **Zero Dependencies**: Single static binary
- **Git LFS Storage**: Efficient handling of large datasets
- **Certificate Authentication**: Secure access control
- **Optional Encryption**: AES-256-GCM for sensitive data
- **GitHub Actions Ready**: Native CI/CD integration

## Repository Structure

```
github-db/                 # Main implementation repository
├── src/                  # Source code
├── template/             # Database template (submodule)
└── setup.sh             # Repository setup script

github-db-template/       # Template repository
├── .github-db/          # Database storage (LFS)
├── data/                # JSON documents
├── certs/               # Certificate storage
└── .github/workflows/   # Automated workflows
```

## Initial Setup

1. Clone and setup repositories:
```bash
# Clone this repository
git clone https://github.com/your-org/github-db
cd github-db

# Run setup script
chmod +x setup.sh
./setup.sh
```

2. Create GitHub repositories:
- Create `github-db` for the main implementation
- Create `github-db-template` for the database template

3. Push repositories:
```bash
# Push main repository
git remote add origin https://github.com/your-org/github-db.git
git push -u origin main

# Push template repository
cd ../github-db-template
git remote add origin https://github.com/your-org/github-db-template.git
git push -u origin main
```

4. Configure GitHub secrets:
- In the main repository:
  * `TEMPLATE_TOKEN`: GitHub token with access to template repository

## Release Process

### Automatic Release

1. Create and push a tag:
```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

### Manual Release

1. Go to Actions → Release Binary → Run workflow
2. Enter version (e.g., v1.0.0)
3. Run workflow

The workflow will:
- Build a static binary
- Create a GitHub release
- Update the template repository
- Generate release notes and checksums

## Creating New Database Instance

1. Create from template:
```bash
gh repo create my-db --template your-org/github-db-template
```

2. Set up required secrets:
```bash
# Generate and add certificate
gh secret set DB_CERT -b"$(cat cert.pem | base64)"

# Optional: Set encryption key
gh secret set DB_KEY -b"your-encryption-key"
```

## Usage Examples

### Basic Operations

```bash
# Create document
github-db -c cert.pem create doc1 '{"name": "Test"}'

# Read document
github-db -c cert.pem read doc1

# Update document
github-db -c cert.pem update doc1 '{"name": "Updated"}'

# Delete document
github-db -c cert.pem delete doc1

# List documents
github-db -c cert.pem list
```

### Using Environment Variables

```bash
export DB_CERT_CONTENT=$(cat cert.pem | base64)
export DB_KEY="your-encryption-key"
export DB_JSON_OUTPUT=true

# Create with stdin
echo '{"name": "Test"}' | github-db --stdin create doc1
```

### File-Based Operations

Place JSON files in the `data/` directory:
```
data/
  └── doc1.json
```

The workflow automatically processes file changes:
- Adding/modifying files creates/updates documents
- Deleting files removes documents

## Security Notes

1. Certificate Management
   - Generate unique certificates for different purposes
   - Store certificates in GitHub Secrets
   - Regularly rotate certificates

2. Encryption
   - Use strong encryption keys (32 bytes)
   - Store keys securely in GitHub Secrets
   - Enable encryption for sensitive data

3. Access Control
   - Use separate certificates for different users/services
   - Revoke certificates when no longer needed
   - Monitor access through Git history

## Development

### Building from Source

```bash
# Build release binary
cargo build --release

# Build static binary
RUSTFLAGS='-C target-feature=+crt-static' \
cargo build --release --target x86_64-unknown-linux-musl
```

### Running Tests

```bash
cargo test
```

## License

MIT License - See [LICENSE](LICENSE) for details
