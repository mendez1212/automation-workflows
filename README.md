# 🛠️ Automation Workflows Repository

This repository contains **automation workflows** for repository management, UI image processing and serverless deployment, built with **Bash**, **Rust** and **Node.js/Vercel**.

---

## 📂 Repository Structure

```

automation-workflows/
├─ scripts/                 # Bash scripts for repo creation
├─ rust-ui-processor/       # Rust workflow for generating UI gallery
├─ js-vercel-processor/     # Node.js workflow deployed on Vercel for GitHub webhooks & image processing
└─ README.md                # top-level overview

```

---

## 🟢 Modules Overview

### 1️⃣ `scripts/`
- **Purpose:** Automate GitHub repository creation.
- **Contents:**
  - `create-repo.sh`: Bash script to create new repositories programmatically.
  - `README.md`: Instructions for usage.
- **Highlights:**
  - Supports organization or user repositories.
  - Public repo creation only; private repos handled manually.
  - Integrates with GitHub API using curl and bash.

---

### 2️⃣ `rust-ui-processor/`
- **Purpose:** Generate and update UI galleries for any repository.
- **Features:**
  - Automated image resizing, gallery generation and README previews.
  - Configurable parameters: image folder, max width, border radius, gallery columns, and fast check mode.
  - Reusable GitHub Actions workflow callable from any repo.
  - Caching and optimized Rust builds for faster CI/CD.
- **Tech Stack:** Rust, GitHub Actions, cargo, Ubuntu runners.

---

### 3️⃣ `js-vercel-processor/`
- **Purpose:** Serverless automation using GitHub App webhooks on Vercel.
- **Features:**
  - Listens to `docs/ui/*.png` pushes.
  - Resizes images to `300px` width and applies `6.5%` rounded corners.
  - Implements LRU caching and concurrency control.
  - Commits processed files back to GitHub automatically.
  - Detailed logging for debugging & analytics.
- **Tech Stack:** Node.js (>=20), Vercel, Sharp, Octokit, raw-body.

---

## ⚡ Features & Advantages
- Cross-language automation: Bash + Rust + Node.js.
- CI/CD-ready: GitHub Actions for Rust workflow, serverless webhooks for Node.js workflow.
- Optimized performance: caching, concurrency, error handling, and retries.
- Modular design: each workflow can run independently or be reused across multiple repos.

---

## 📝 Notes
- All workflows respect caching to avoid unnecessary processing.
- Logs provide transparent insights for debugging and performance metrics.
- Designed for maintainability and scalability in multi-repo environments.

---

## 📌 Links & References
- JS Vercel Processor: [`js-vercel-processor/README.md`](js-vercel-processor/README.md)
- Rust UI Processor: [`rust-ui-processor/README.md`](rust-ui-processor/README.md)
- Bash Scripts: [`scripts/README.md`](scripts/README.md)

---



#### ℹ️ this repository contains selected folders and projects from my organization work.    
> Some commits have been hidden to protect sensitive information.  
> The commit count and dates are preserved to reflect my contributions and development timeline.

---
> **Repository created on:** 2025-09-11, 08:46 (UTC+3)
