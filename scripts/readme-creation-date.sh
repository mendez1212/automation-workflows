#!/bin/bash

ORG="falconsoft25"

echo "🔑 Authenticating with GitHub CLI… Shera"
gh auth login

echo "🕵️ Checking repositories in $ORG… Shera"
repos=$(gh repo list "$ORG" --limit 1000 --json name --jq '.[].name')

for repo in $repos; do
  echo "🔍 Processing repo: $repo… Shera"
  
  # Get repo creation date
  created_at=$(gh api "repos/$ORG/$repo" --jq '.created_at')
  created_at_formatted=$(date -d "$created_at" "+%Y-%m-%d, %H:%M (UTC%:::z)")
  
  # Clone repo shallowly
  git clone --depth=1 "https://github.com/$ORG/$repo.git" tmp_repo 2>/dev/null
  
  cd tmp_repo || exit
  
  if [ ! -f README.md ]; then
    echo "🚨 No README.md found in $repo… creating it, Shera"
    echo "# $repo" > README.md
  fi
  
  if ! grep -q "Repository created on:" README.md; then
    echo "" >> README.md
    echo "> **Repository created on:** $created_at_formatted" >> README.md
    echo "📄 Added creation date to README.md in $repo… Shera"
    
    # Commit & push changes
    git config user.name "AutoBot-Shera"
    git config user.email "autobot@example.com"
    git add README.md
    git commit -m "Add repo creation date to README.md"
    git push origin main || git push origin master
  else
    echo "✅ README.md already has creation date in $repo… Shera"
  fi

  cd ..
  rm -rf tmp_repo
done

echo "🎯 All done, Shera."
