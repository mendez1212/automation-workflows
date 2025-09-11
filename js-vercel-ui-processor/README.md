# 🌐 JS Vercel Processor

A **serverless Node.js application** deployed on **Vercel** that listens to **GitHub webhooks** and automatically processes UI images (`docs/ui/*.png`) on push events.

---

## 🚀 What It Does
- **Listens to GitHub webhooks** via a Vercel serverless function.
- **Validates signatures** with `@octokit/webhooks` for security.
- **Fetches changed files** from GitHub using an **App Installation token**.
- **Optimizes PNG images**:
  - Resizes to `300px` width.
  - Applies a **6.5% rounded corner radius**.
  - Skips already-optimized images (uses caching).
- **Commits optimized files back** to the repo automatically.
- **Concurrency control** for safe batch processing on serverless environments.

---

## 🛠️ Tech Stack
- **Vercel Serverless** (Node.js runtime)
- **GitHub App** (Octokit + App Auth)
- **Image processing** with [`sharp`](https://sharp.pixelplumbing.com/)
- **Caching** (LRU-based for processed files & SVG masks)
- **Security** via signature validation (`GITHUB_WEBHOOK_SECRET`)

---

## 📂 Repo Structure
```

js-vercel-processor/
├─ api/
│  └─ index.mjs        # Webhook handler & image processor
├─ .gitignore          # ignore pushing unecessary files
├─ package-lock.json   # Dependency lock file
├─ package.json        # Dependencies & metadata
├─ README.md           # Documentation
└─ vercel.json         # Vercel deployment config


```

---

## 🔑 Required Environment Variables
Set these in Vercel project settings:

| Variable              | Description                                         |
|------------------------|-----------------------------------------------------|
| `GITHUB_WEBHOOK_SECRET` | Secret used to verify webhook payloads              |
| `APP_ID`               | GitHub App ID                                       |
| `PRIVATE_KEY`          | GitHub App private key (PEM format)                 |

---

## 📌 Example Workflow
1. Developer pushes `docs/ui/*.png` changes to `main`.
2. GitHub sends a webhook → received by **Vercel function**.
3. Function:
   - Fetches changed files.
   - Runs **check + resize + rounded corners**.
   - Commits changes back to repo.
4. Logs show **processed/skipped/cached** counts.

---

## ⚡ Features
- **Early exit logic** → skips non-PNG or already-optimized files.
- **Batch processing** with **concurrency limits** (`2` in serverless).
- **Exponential backoff retries** for GitHub API calls.
- **Cache system** for:
  - Processed file SHAs.
  - SVG mask reuse (radius/width combos).
- **Detailed logging** for debugging & performance insights.

---

## 📊 Example Logs
```

🔔 Webhook received: Push to falcon12120/repo-name/main
📋 Found 3 PNG files to process: \[docs/ui/login.png, docs/ui/dashboard.png, docs/ui/card.png]
🚀 Starting batch processing of 3 files...
✅ Successfully processed and committed dashboard.png in 230ms
✓ Skipping card.png: already meets requirements
📊 Summary: Processed=1, Skipped=1, Cached=1, Failed=0

```

---

## 🧑‍💻 Notes
This project demonstrates:
- Building **secure GitHub Apps** with webhook handling.
- Deploying **serverless automation tools** on Vercel.
- Writing **production-grade Node.js** with caching, retries and error handling.
- Applying **DevOps + frontend automation** in a cross-platform ecosystem.

---