#!/bin/bash

# GitHub user
USER="falcon12120"
TOKEN=$(gh auth token)  # 🔐 Uses your active gh CLI session

# --- Parse Arguments ---
REPO_NAME=$(echo "$1" | tr '[:upper:]' '[:lower:]')
shift

TOPICS_INPUT=$(echo "$1" | tr '[:upper:]' '[:lower:]')
shift

if [ $# -eq 1 ]; then
  if [[ "$1" == *" "* ]]; then
    REPO_DESCRIPTION=$(echo "$1" | tr '[:upper:]' '[:lower:]')
  else
    echo "❌ ERROR: Repo description must be enclosed in double quotes AND contain a space."
    echo "Example: createrepo repo-name topics_input \"repo description\""
    exit 1
  fi
elif [ $# -gt 0 ]; then
  echo "❌ ERROR: Too many arguments or improperly quoted repo description."
  echo "Usage: createrepo <repo_name> <topics_input_without_space_use '- and _'> \"<repo_description>\""
  exit 1
else
  REPO_DESCRIPTION=""
fi

# --- Check if Repository Already Exists ---
EXISTING_REPO=$(gh repo view "$USER/$REPO_NAME" --json name -q .name 2>/dev/null)
if [ "$EXISTING_REPO" == "$REPO_NAME" ]; then
  echo "❌ ERROR: Repository '$REPO_NAME' already exists for user '$USER'."
  exit 1
fi

# --- Create GitHub Repo (public) ---
curl -s -X POST "https://api.github.com/user/repos" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Accept: application/vnd.github+json" \
  -d @- <<EOF
{
  "name": "$REPO_NAME",
  "description": "$REPO_DESCRIPTION",
  "private": false
}
EOF

# --- Set GitHub Topics from TOPICS_INPUT ---
TOPICS=$(echo "$TOPICS_INPUT" | tr '_' '\n' | awk '{print "\"" $0 "\""}' | paste -sd, -)
curl -s -X PUT "https://api.github.com/repos/$USER/$REPO_NAME/topics" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Accept: application/vnd.github+json" \
  -d "{\"names\": [$TOPICS]}"

# --- Add COPYRIGHT.md file ---
COPYRIGHT_MD_CONTENT="## Copyright © $(date +%Y) $USER

All rights reserved.

This source code is **proprietary** and protected by international copyright law.

Any reproduction, distribution, modification or unauthorized use in whole or in part is **strictly prohibited**.

Violators will face legal consequences, including but not limited to:
- 🚫 DMCA takedowns  
- 🔒 Permanent bans from platforms  
- ⚖️ Prosecution to the fullest extent of the law"
ENCODED_COPYRIGHT_MD_CONTENT=$(echo "$COPYRIGHT_MD_CONTENT" | base64)

curl -s -X PUT "https://api.github.com/repos/$USER/$REPO_NAME/contents/COPYRIGHT.md" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Accept: application/vnd.github+json" \
  -d @- <<EOF
{
  "message": "added COPYRIGHT.md",
  "content": "$ENCODED_COPYRIGHT_MD_CONTENT"
}
EOF

# --- Prepare README.md content ---
CREATED_AT=$(date -u +"%Y-%m-%d, %H:%M" -d '+3 hours')
README_CONTENT="---
> **Repository created on:** $CREATED_AT (UTC+3)"
ENCODED_README_CONTENT=$(echo "$README_CONTENT" | base64)

curl -s -X PUT "https://api.github.com/repos/$USER/$REPO_NAME/contents/README.md" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Accept: application/vnd.github+json" \
  -d @- <<EOF
{
  "message": "added README.md",
  "content": "$ENCODED_README_CONTENT"
}
EOF

# --- Create docs/ui-gallery.md ---
UI_GALLERY_CONTENT="# This document contains all UI screenshots.\n"
ENCODED_UI_GALLERY_CONTENT=$(echo "$UI_GALLERY_CONTENT" | base64)

curl -s -X PUT "https://api.github.com/repos/$USER/$REPO_NAME/contents/docs/ui-gallery.md" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Accept: application/vnd.github+json" \
  -d @- <<EOF
{
  "message": "added docs/ui-gallery.md",
  "content": "$ENCODED_UI_GALLERY_CONTENT"
}
EOF

# --- Create docs/ui/.keep ---
KEEP_FILE_CONTENT="(placeholder to keep the UI folder)"
ENCODED_KEEP_FILE=$(echo "$KEEP_FILE_CONTENT" | base64)

curl -s -X PUT "https://api.github.com/repos/$USER/$REPO_NAME/contents/docs/ui/.keep" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Accept: application/vnd.github+json" \
  -d @- <<EOF
{
  "message": "added docs/ui/.keep",
  "content": "$ENCODED_KEEP_FILE"
}
EOF

# --- Done ---
echo ""
echo "✅ Created public repo '$REPO_NAME' for user '$USER'"
echo "Description: $REPO_DESCRIPTION"
echo -n "Topics: "
for topic in $(echo "$TOPICS_INPUT" | tr '_' ' '); do
  echo -n "🔹$topic | "
done
echo ""
echo "📄 docs/ui added"
echo "📄 docs/ui-gallery.md added"
echo "📄 COPYRIGHT.md added"
echo "📄 README.md added"