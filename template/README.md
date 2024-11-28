# GitHub DB Instance

A secure, Git LFS-backed JSON database instance with multiple data input methods.

## Setup

1. Create a new repository from this template
2. Add required secrets:
   - `DB_CERT`: Your database certificate (base64 encoded)
   - `DB_KEY`: (Optional) Encryption key for sensitive data

## Data Input Methods

### 1. File-Based Operations

Place JSON files in the `data/` directory:
```
data/
  ├── document1.json
  ├── document2.json
  └── example.json
```

The workflow automatically:
- Creates/updates documents when JSON files are added/modified
- Deletes documents when files are removed
- Uses filenames (without .json) as document IDs

Example file (data/example.json):
```json
{
  "name": "Example Document",
  "description": "This is an example document",
  "metadata": {
    "created": "2024-01-01",
    "tags": ["example", "demo"]
  }
}
```

### 2. Workflow Dispatch

Use GitHub Actions UI or API to perform operations:

```bash
# Create document
gh workflow run database.yml -f operation=create -f id=doc1 -f data='{"name": "Test"}'

# Read document
gh workflow run database.yml -f operation=read -f id=doc1

# Update document
gh workflow run database.yml -f operation=update -f id=doc1 -f data='{"name": "Updated"}'

# Delete document
gh workflow run database.yml -f operation=delete -f id=doc1

# List all documents
gh workflow run database.yml -f operation=list
```

### 3. API Integration

Use GitHub's API to trigger workflow:

```bash
curl -X POST \
  -H "Authorization: token ${GITHUB_TOKEN}" \
  -H "Accept: application/vnd.github.v3+json" \
  https://api.github.com/repos/owner/repo/actions/workflows/database.yml/dispatches \
  -d '{
    "ref": "main",
    "inputs": {
      "operation": "create",
      "id": "doc1",
      "data": "{\"name\": \"Test\"}"
    }
  }'
```

## Security Features

1. Certificate Authentication
   - All operations require valid certificate
   - Certificate stored in repository secrets
   - Automatic validation for each operation

2. Data Encryption
   - Optional AES-256-GCM encryption
   - Encryption key in repository secrets
   - Transparent encryption/decryption

3. ID Uniqueness
   - Automatic validation of document IDs
   - Prevents duplicate documents
   - Case-sensitive IDs

## Example Operations

### Create and Update Cycle

1. Create document:
   ```json
   // data/user123.json
   {
     "name": "John Doe",
     "email": "john@example.com"
   }
   ```

2. Update document:
   ```json
   // Modified data/user123.json
   {
     "name": "John Doe",
     "email": "john@example.com",
     "status": "active"
   }
   ```

3. Delete document:
   ```bash
   # Simply delete data/user123.json
   rm data/user123.json
   git commit -am "Remove user123" && git push
   ```

### Batch Operations

Create multiple documents:
```bash
for i in {1..3}; do
  echo "{\"index\": $i}" > "data/batch$i.json"
done
git add data/
git commit -m "Add batch documents"
git push
```

## Workflow Features

1. Automatic Validation
   - JSON syntax checking
   - ID uniqueness enforcement
   - Certificate verification

2. Operation Verification
   - Confirms successful operations
   - Validates data integrity
   - Reports detailed errors

3. Git LFS Management
   - Automatic LFS tracking
   - Efficient storage handling
   - Backup management
