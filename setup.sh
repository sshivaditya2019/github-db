#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${GREEN}Setting up GitHub DB repositories...${NC}"

# Setup main repository
echo "Initializing main repository..."
git init
git add .
git commit -m "Initial commit: GitHub DB implementation"

# Create template repository
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

# Return to main repository
cd ../github-db

# Add template as a submodule
git submodule add ../github-db-template template

echo -e "${GREEN}Setup complete!${NC}"
echo -e "Next steps:"
echo -e "1. Create a new repository on GitHub for github-db"
echo -e "2. Create a new repository on GitHub for github-db-template"
echo -e "3. Push both repositories:"
echo -e "\nFor main repository:"
echo -e "  git remote add origin https://github.com/your-org/github-db.git"
echo -e "  git push -u origin main"
echo -e "\nFor template repository:"
echo -e "  cd ../github-db-template"
echo -e "  git remote add origin https://github.com/your-org/github-db-template.git"
echo -e "  git push -u origin main"
echo -e "\nTo use the template:"
echo -e "  gh repo create my-db --template your-org/github-db-template"
