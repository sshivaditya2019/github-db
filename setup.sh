#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${GREEN}Setting up GitHub DB repositories...${NC}"

# Get GitHub username
read -p "Enter your GitHub username: " GITHUB_USER

# Setup template repository first
echo "Creating template repository..."
mkdir -p ../github-db-template
cp -r template/* ../github-db-template/
cp template/.gitattributes ../github-db-template/
cd ../github-db-template

# Initialize template repository
git init
git lfs install
git add .
git commit -m "Initial commit: Database template"

echo "Template repository created at ../github-db-template"

# Return to main repository
cd ../github-db

# Remove existing template directory
rm -rf template

# Update .gitmodules with correct URL
cat > .gitmodules << EOL
[submodule "template"]
    path = template
    url = https://github.com/${GITHUB_USER}/github-db-template.git
    branch = main
EOL

# Initialize repository
git init
git add .
git commit -m "Initial commit: GitHub DB implementation"

# Initialize submodule
git submodule add https://github.com/${GITHUB_USER}/github-db-template.git template

echo -e "${GREEN}Setup complete!${NC}"
echo -e "\nNext steps:"
echo -e "1. Create github-db-template repository on GitHub:"
echo -e "   gh repo create github-db-template --public"
echo -e "   cd ../github-db-template"
echo -e "   git remote add origin https://github.com/${GITHUB_USER}/github-db-template.git"
echo -e "   git push -u origin main"
echo -e "\n2. Create github-db repository on GitHub:"
echo -e "   cd ../github-db"
echo -e "   gh repo create github-db --public"
echo -e "   git remote add origin https://github.com/${GITHUB_USER}/github-db.git"
echo -e "   git push -u origin main"
echo -e "\n3. Add TEMPLATE_TOKEN secret to github-db repository"
echo -e "   This token needs write access to github-db-template repository"
echo -e "\n4. Create a release:"
echo -e "   git tag -a v0.1.0 -m \"Initial release\""
echo -e "   git push origin v0.1.0"
echo -e "\nTo create a new database instance:"
echo -e "   gh repo create my-db --template ${GITHUB_USER}/github-db-template"
