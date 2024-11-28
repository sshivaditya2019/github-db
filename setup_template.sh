#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Get GitHub username
read -p "Enter your GitHub username: " GITHUB_USER

echo -e "${GREEN}Setting up template repository...${NC}"

# Check if repository exists
REPO_EXISTS=$(gh repo view "${GITHUB_USER}/github-db-template" 2>/dev/null || echo "false")

if [ "$REPO_EXISTS" != "false" ]; then
    echo -e "${YELLOW}Repository github-db-template already exists${NC}"
    read -p "Do you want to reset it? (y/N) " RESET_REPO
    if [[ $RESET_REPO =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Resetting repository...${NC}"
        gh repo delete "${GITHUB_USER}/github-db-template" --yes || true
        gh repo create github-db-template --public --confirm
    else
        echo -e "${YELLOW}Using existing repository...${NC}"
        # Clean up existing template directory if it exists
        if [ -d "../github-db-template" ]; then
            rm -rf "../github-db-template"
        fi
    fi
else
    # Create new repository
    echo -e "\n${GREEN}Creating github-db-template repository...${NC}"
    gh repo create github-db-template --public --confirm || true
fi

# Create temporary directory for template
echo -e "\n${GREEN}Setting up template files...${NC}"
mkdir -p ../github-db-template
cd ../github-db-template

# Initialize Git and LFS
git init
git lfs install

# Create directory structure first
mkdir -p .github/workflows
mkdir -p .github-db
mkdir -p data
mkdir -p certs
mkdir -p backups
touch .github-db/.gitkeep
touch certs/.gitkeep
touch backups/.gitkeep

# Copy template files
cp -r ../github-db/template/* .
cp ../github-db/template/.gitattributes .
cp ../github-db/template/.gitignore .

# Ensure workflows directory exists and copy workflow
mkdir -p .github/workflows
cp ../github-db/template/.github/workflows/database.yml .github/workflows/

# Update workflow file with correct owner
echo -e "\n${GREEN}Updating workflow configuration...${NC}"
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS version of sed
    sed -i '' "s/OWNER/${GITHUB_USER}/g" .github/workflows/database.yml
else
    # Linux version of sed
    sed -i "s/OWNER/${GITHUB_USER}/g" .github/workflows/database.yml
fi

# Add encryption and certificate configuration
echo -e "\n${GREEN}Setting up security configuration...${NC}"

# Update .gitignore for security
cat >> .gitignore << EOL
# Security files
certs/*
!certs/.gitkeep
*.key
*.cert
.env
EOL

# Create example environment file
cat > .env.example << EOL
# Database encryption key (32 bytes hex)
DB_KEY=

# Certificate content (base64 encoded)
DB_CERT=
EOL

# Initialize Git LFS tracking
git lfs track ".github-db/**"
git lfs track "*.blob"

# Create README with security instructions
cat > README.md << EOL
# GitHub Database

Secure document database using GitHub as storage.

## Setup

1. Generate encryption key:
\`\`\`bash
openssl rand -hex 32 > .env
echo "DB_KEY=\$(cat .env)" >> .env
\`\`\`

2. Generate certificate:
\`\`\`bash
# Download latest binary
curl -L -o github-db \\
  https://github.com/${GITHUB_USER}/github-db/releases/latest/download/github-db-linux-x86_64
chmod +x github-db

# Generate certificate
source .env
./github-db --key "\$DB_KEY" generate-cert admin -o ./certs
echo "DB_CERT=\$(cat certs/admin.cert | base64)" >> .env
\`\`\`

3. Set up GitHub secrets:
\`\`\`bash
# Set secrets from .env
gh secret set DB_KEY < .env
gh secret set DB_CERT < .env
\`\`\`

## Usage

1. Add documents:
\`\`\`bash
echo '{"name": "test"}' > data/doc1.json
git add data/doc1.json
git commit -m "Add doc1"
git push
\`\`\`

2. Use CLI:
\`\`\`bash
# Load environment
source .env

# Execute commands
./github-db --key "\$DB_KEY" --cert ./certs/admin.cert list
./github-db --key "\$DB_KEY" --cert ./certs/admin.cert create doc2 '{"name": "test2"}'
\`\`\`

## Security

- Keep your .env file secure and never commit it
- Store encryption key and certificates safely
- Use separate certificates for different users
- Regularly rotate certificates
EOL

# Check if we're using existing repository
if [ "$REPO_EXISTS" != "false" ] && [[ ! $RESET_REPO =~ ^[Yy]$ ]]; then
    # Fetch existing repository
    git fetch origin
    git reset --hard origin/main
    
    # Apply new changes
    echo -e "\n${GREEN}Updating existing repository...${NC}"
    git add .
    git commit -m "Update template configuration" || echo -e "${YELLOW}No changes to commit${NC}"
else
    # Add all files and commit
    echo -e "\n${GREEN}Committing files...${NC}"
    git add .
    git commit -m "Initial template setup"
    git branch -M main
fi

# Push to GitHub
echo -e "\n${GREEN}Pushing to GitHub...${NC}"
if [ "$REPO_EXISTS" != "false" ] && [[ ! $RESET_REPO =~ ^[Yy]$ ]]; then
    git push --force-with-lease origin main
else
    git remote add origin "https://github.com/${GITHUB_USER}/github-db-template.git"
    git push -u origin main
fi

echo -e "\n${GREEN}Template repository setup complete!${NC}"
echo -e "\nNext steps:"
echo -e "1. Go to https://github.com/${GITHUB_USER}/github-db-template/settings"
echo -e "2. Check 'Template repository' under General settings"
echo -e "\nTo test the template:"
echo -e "gh repo create test-db --template ${GITHUB_USER}/github-db-template"
echo -e "\nThen follow the setup instructions in the README.md file to:"
echo -e "1. Generate an encryption key"
echo -e "2. Create certificates"
echo -e "3. Set up GitHub secrets"
