# 🛠️ Automation Scripts

This folder contains Bash scripts for automating GitHub repository tasks under the user or organization account. These scripts demonstrate expertise in **GitHub CLI/API**, automation, workflow management, and scripting best practices.

---

## 💾 Scripts Included

### 1. `create-repo.sh` – GitHub Repo Creator
- **Purpose:** Automatically creates a new **public repository** with a README, `COPYRIGHT.md`, and `docs/ui` folder structure.
- **Features:**
  - Creates repository using GitHub API + `curl`.
  - Adds `README.md` with creation date (UTC+3).
  - Adds `COPYRIGHT.md` with legal boilerplate.
  - Adds `docs/ui-gallery.md` for UI screenshots.
  - Adds placeholder `docs/ui/.keep` to preserve folder structure.
  - Supports **topics** and custom **description** via command-line arguments.
- **Usage Example:**
  ```bash
  curl -s https://raw.githubusercontent.com/Falcon12120/automation-workflows/main/scripts/create-repo.sh | bash -s -- repo-name topics_input "description"
  ```
- **Tech:** Bash, GitHub API, Git, CLI automation

---

### 2. `readme-creation-date.sh` – Repo Creation Date Inserter
- **Purpose:** Updates all repositories in an **organization** to include their creation date in the top-level README.md.
- **Features:**
  - Authenticates with GitHub CLI (`gh`).
  - Lists all repositories under an organization.
  - Clones each repository shallowly.
  - Creates or updates README.md with formatted creation date.
  - Commits and pushes changes automatically.
- **Demo Entry Added to README.md:**
  > **Repository created on:** 2025-06-10, 22:40 (UTC+3)
  - **Usage Example:**
  Unlike `create-repo.sh` (which works fully through GitHub API calls),  
  this script **must be downloaded to your PC and run locally** because it uses Git operations.  

  Run it like this:
  ```bash
  bash readme-creation-date.sh
- **Tech:** Bash, GitHub CLI (`gh`), Git, date/time formatting

---

## ⚡️ Highlights
- Shows **end-to-end automation** across multiple repos.
- Demonstrates **scripted Git operations**, API usage, and workflow integration.
- Ready for use as part of CI/CD pipelines or portfolio demos.
- Cleanly preserves folder structures for documentation and UI galleries.

---

## 📌 Notes for Recruiters
- These scripts are stored inside GitHub but can be run via the command-line interface (CLI) on your own computer.
- Sensitive data like tokens are handled via CLI session (`gh auth token`) for security.
- Excellent example of **DevOps, CI/CD, and automation skillset** in action.
---
