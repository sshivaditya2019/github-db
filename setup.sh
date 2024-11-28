#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${GREEN}Setting up GitHub DB repositories...${NC}"

# Get GitHub username
read -p "Enter your GitHub username: " GITHUB_USER

# Create and setup template repository first
echo "Creating template repository..."
gh repo create github-db-template --public --confirm || true

# Setup template repository
echo "Setting up template repository..."
mkdir -p ../github-db-template
cp -r template/* ../github-db-template/
cp template/.gitattributes ../github-db-template/
cd ../github-db-template

# Initialize template repository
git init
git lfs install
git add .
git commit -m "Initial commit: Database template"
git branch -M main
git remote add origin "https://github.com/${GITHUB_USER}/github-db-template.git"
git push -u origin main

# Return to main repository
cd ../github-db

# Create main repository
echo "Creating main repository..."
gh repo create github-db --public --confirm || true

# Clean up existing files
rm -rf template
rm -f .gitmodules

# Initialize main repository
git init
git add .
git commit -m "Initial commit: Core implementation"
git branch -M main
git remote add origin "https://github.com/${GITHUB_USER}/github-db.git"

# Add template as submodule
echo "Adding template as submodule..."
cat > .gitmodules << EOL
[submodule "template"]
    path = template
    url = https://github.com/${GITHUB_USER}/github-db-template.git
    branch = main
EOL

git submodule add "https://github.com/${GITHUB_USER}/github-db-template.git" template
git add .gitmodules template
git commit -m "Add template submodule"
git push -u origin main

echo -e "${GREEN}Setup complete!${NC}"
echo -e "\nNext steps:"
echo -e "1. Add TEMPLATE_TOKEN to github-db repository:"
echo -e "   - Go to GitHub Settings → Developer settings → Personal access tokens"
echo -e "   - Generate new token with 'repo' scope"
echo -e "   - Run: gh secret set TEMPLATE_TOKEN -b\"your_token_here\""
echo -e "\n2. Create initial release:"
echo -e "   git tag -a v0.1.0 -m \"Initial release\""
echo -e "   git push origin v0.1.0"
echo -e "\nTo create a new database instance:"
echo -e "   gh repo create my-db --template ${GITHUB_USER}/github-db-template"
