# Setting Up Template Repository

1. Create new repository for the template:
```bash
# Create the template repository on GitHub
gh repo create github-db-template --public

# Create temporary directory for template
mkdir ../github-db-template
cd ../github-db-template

# Initialize Git and LFS
git init
git lfs install

# Copy template files from main repo
cp -r ../github-db/template/* .
cp ../github-db/template/.gitattributes .
cp ../github-db/template/.gitignore .

# Initialize directories
mkdir -p .github-db
mkdir -p certs
touch .github-db/.gitkeep
touch certs/.gitkeep

# Add certs directory to gitignore
echo "certs/*" >> .gitignore
echo "!certs/.gitkeep" >> .gitignore

# Update workflow file to use correct owner
sed -i '' "s/OWNER/$YOUR_GITHUB_USERNAME/g" .github/workflows/database.yml

# Commit and push
git add .
git commit -m "Initial template setup"
git branch -M main
git remote add origin https://github.com/$YOUR_GITHUB_USERNAME/github-db-template.git
git push -u origin main
```

2. Make repository a template:
- Go to GitHub repository settings
- Under "General"
- Check "Template repository"

3. Test template with encryption:
```bash
# Create test repository from template
gh repo create test-db --template $YOUR_GITHUB_USERNAME/github-db-template

# Clone and set up test repository
git clone https://github.com/$YOUR_GITHUB_USERNAME/test-db.git
cd test-db

# Download github-db binary
curl -L -o github-db \
  https://github.com/$YOUR_GITHUB_USERNAME/github-db/releases/latest/download/github-db-linux-x86_64
chmod +x github-db

# Generate encryption key and certificate
ENCRYPTION_KEY=$(openssl rand -hex 32)
./github-db --key "$ENCRYPTION_KEY" generate-cert test-cert -o ./certs

# Set up GitHub secrets
gh secret set DB_CERT -b"$(cat certs/test-cert.cert | base64)"
gh secret set DB_KEY -b"$ENCRYPTION_KEY"

# Test database operations
./github-db --key "$ENCRYPTION_KEY" --cert ./certs/test-cert.cert create test '{"name": "test"}'

# Verify directory structure:
# - .github/workflows/database.yml
# - .github-db/
# - certs/
# - data/example.json
# - README.md
# - .gitattributes
# - .gitignore
```

4. Clean up test:
```bash
# Delete test repository
gh repo delete test-db --yes
```

## Template Structure

The template includes:

1. Core Directories:
   - `.github-db/`: Database storage directory
   - `certs/`: Certificate storage directory (gitignored)
   - `data/`: JSON document directory
   - `.github/workflows/`: GitHub Actions workflows

2. Configuration Files:
   - `.gitattributes`: Git LFS configuration
   - `.gitignore`: Ignores sensitive files (certs, encryption keys)
   - `README.md`: Basic usage instructions

3. Example Files:
   - `data/example.json`: Example document format
   - `.github/workflows/database.yml`: GitHub Actions workflow

## Security Features

The template is configured for secure operation:

1. Certificate Management:
   - `certs/` directory for storing encrypted certificates
   - Gitignored to prevent accidental commits
   - Supports encrypted certificate storage

2. GitHub Actions:
   - Workflow configured for secure operations
   - Uses repository secrets for certificates and keys
   - Supports encrypted database operations

3. Access Control:
   - Certificate-based authentication
   - Encryption key support
   - Secure secret management

The template repository is now ready for users to create new encrypted databases with proper certificate management.
