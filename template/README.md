# GitHub DB Instance

A secure Git LFS-backed JSON database instance.

## Setup

1. Generate certificate:
```bash
# Download the binary
curl -L -o github-db \
  https://github.com/OWNER/github-db/releases/latest/download/github-db-linux-x86_64
chmod +x github-db

# Generate certificate
./github-db generate-cert my-cert -o ./certs
```

2. Add secrets to repository:
```bash
# Add certificate (required)
gh secret set DB_CERT -b"$(cat certs/my-cert.cert | base64)"

# Optional: Add encryption key
gh secret set DB_KEY -b"$(openssl rand -base64 32)"
```

## Usage

### File-Based Operations

Simply add, modify, or delete JSON files in the `data/` directory:

```bash
# Create/Update document
echo '{"name": "test"}' > data/doc1.json
git add data/doc1.json
git commit -m "Add doc1"
git push

# Delete document
rm data/doc1.json
git commit -m "Delete doc1" data/doc1.json
git push
```

### Using GitHub Actions

Use the workflow dispatch to perform operations:

```bash
# Create document
gh workflow run database.yml -f operation=create -f id=doc2 -f data='{"name": "test2"}'

# Read document
gh workflow run database.yml -f operation=read -f id=doc2

# Update document
gh workflow run database.yml -f operation=update -f id=doc2 -f data='{"name": "updated"}'

# Delete document
gh workflow run database.yml -f operation=delete -f id=doc2

# List all documents
gh workflow run database.yml -f operation=list
```

## Directory Structure

```
.
├── .github-db/  # Database storage (Git LFS)
├── data/       # JSON documents
└── certs/      # Certificate storage (local only)
```

## Security

- All operations require valid certificate
- Optional encryption for sensitive data
- Git LFS for efficient storage
- Full audit trail through Git history

## Best Practices

1. Certificate Management
   - Keep private key secure (never commit to repo)
   - Store certificate in GitHub Secrets
   - Generate new certificate if compromised

2. Data Organization
   - Use meaningful document IDs
   - Keep JSON files in data/ directory
   - Use consistent schema

3. Encryption
   - Enable encryption for sensitive data
   - Store encryption key securely
   - Never commit encryption key

## Maintenance

The workflow automatically:
- Processes file changes in data/
- Handles CRUD operations
- Maintains Git LFS storage
- Provides audit trail

## Troubleshooting

1. Certificate Issues:
   - Verify DB_CERT is properly base64 encoded
   - Check certificate permissions
   - Regenerate certificate if needed

2. File Operations:
   - Ensure JSON is valid
   - Check file permissions
   - Verify Git LFS is working

3. Workflow Issues:
   - Check workflow permissions
   - Verify secrets are set
   - Look at workflow logs
