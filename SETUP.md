# Setup Instructions

## Initial Setup (For Maintainer)

1. Create both repositories on GitHub:
```bash
# Create the template repository first
gh repo create github-db-template --public --clone
cd github-db-template
git lfs install
cp -r /path/to/original/template/* .
git add .
git commit -m "Initial template setup"
git push -u origin main
cd ..

# Create the main repository
gh repo create github-db --public --clone
cd github-db
```

2. Set up the main repository:
```bash
# Copy all source files except template/
cp -r /path/to/original/{src,Cargo.toml,.github,.gitignore,.gitattributes} .

# Initialize with submodule
git init
git submodule add https://github.com/YOUR_USERNAME/github-db-template.git template
git add .
git commit -m "Initial commit"
git push -u origin main
```

3. Create a Personal Access Token:
- Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
- Generate new token with `repo` scope
- Add token as repository secret:
```bash
gh secret set TEMPLATE_TOKEN -b"your_token_here"
```

4. Create first release:
```bash
git tag -a v0.1.0 -m "Initial release"
git push origin v0.1.0
```

## Using the Database (For Users)

1. Create new database from template:
```bash
gh repo create my-database --template MAINTAINER_USERNAME/github-db-template
cd my-database
```

2. Generate certificate:
```bash
# Download latest binary
curl -L -o github-db \
  https://github.com/MAINTAINER_USERNAME/github-db/releases/latest/download/github-db-linux-x86_64
chmod +x github-db

# Generate certificate
./github-db generate-cert my-cert -o ./certs
```

3. Set up repository secrets:
```bash
# Add certificate (required)
gh secret set DB_CERT -b"$(cat certs/my-cert.cert | base64)"

# Add encryption key (optional but recommended)
gh secret set DB_KEY -b"$(openssl rand -base64 32)"
```

4. Start using the database:

A. Using data files:
```bash
# Add document
echo '{"name": "test"}' > data/doc1.json
git add data/doc1.json
git commit -m "Add doc1"
git push
```

B. Using GitHub Actions:
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

## Troubleshooting

1. If submodule update fails:
```bash
# Fix submodule URL
git submodule sync
git submodule update --init --force
```

2. If release workflow fails:
- Ensure TEMPLATE_TOKEN is set correctly
- Verify both repositories exist and are accessible
- Check repository permissions

3. If certificate verification fails:
- Regenerate certificate
- Ensure DB_CERT is base64 encoded
- Update repository secret

4. If encryption fails:
- Ensure DB_KEY is exactly 32 bytes
- Update repository secret
