#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

# Get GitHub username
read -p "Enter your GitHub username: " GITHUB_USER

echo -e "${GREEN}Setting up template repository...${NC}"

# Create template repository
echo -e "\n${GREEN}Creating github-db-template repository...${NC}"
gh repo create github-db-template --public --confirm || true

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

# Initialize Git LFS tracking
git lfs track ".github-db/**"
git lfs track "*.blob"

# Add all files and commit
echo -e "\n${GREEN}Committing files...${NC}"
git add .
git commit -m "Initial template setup"
git branch -M main

# Push to GitHub
echo -e "\n${GREEN}Pushing to GitHub...${NC}"
git remote add origin "https://github.com/${GITHUB_USER}/github-db-template.git"
git push -u origin main

echo -e "\n${GREEN}Template repository setup complete!${NC}"
echo -e "\nNext steps:"
echo -e "1. Go to https://github.com/${GITHUB_USER}/github-db-template/settings"
echo -e "2. Check 'Template repository' under General settings"
echo -e "\nTo test the template:"
echo -e "gh repo create test-db --template ${GITHUB_USER}/github-db-template"
