# Setup Instructions

## For Maintainers

1. Create repositories:
```bash
# Create both repositories
gh repo create github-db --public
gh repo create github-db-template --public
```

2. Run setup script:
```bash
./setup.sh
# Enter your GitHub username when prompted
```

3. Create first release:
```bash
git tag -a v0.1.0 -m "Initial release"
git push origin v0.1.0
```

## For Users

1. Create new database from template:
```bash
# Create repository from template
gh repo create my-database --template OWNER/github-db-template
cd my-database
```

2. Generate certificate:
```bash
# Download latest binary
curl -L -o github-db \
  https://github.com/OWNER/github-db/releases/latest/download/github-db-linux-x86_64
chmod +x github-db

# Generate certificate
./github-db generate-cert my-cert -o ./certs
```

3. Set up repository secrets:
```bash
# Add certificate (required)
gh secret set DB_CERT -b"$(cat certs/my-cert.cert | base64)"

# Add encryption key (optional)
gh secret set DB_KEY -b"$(openssl rand -base64 32)"
```

4. Start using the database:
```bash
# Add documents through data/ directory
echo '{"name": "test"}' > data/doc1.json
git add data/doc1.json
git commit -m "Add doc1"
git push
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

## Troubleshooting

1. If workflow fails:
- Check certificate is properly base64 encoded
- Verify DB_CERT secret is set
- Ensure workflow has proper permissions

2. If encryption fails:
- Verify DB_KEY is exactly 32 bytes
- Update DB_KEY secret if needed

3. If file sync fails:
- Check Git LFS is properly configured
- Verify file permissions
- Ensure JSON is valid
