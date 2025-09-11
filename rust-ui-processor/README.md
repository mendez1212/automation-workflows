# ⚡ Rust UI Processor

A **Rust-powered GitHub Actions workflow** for automating UI image processing across repositories.  
It optimizes PNGs, updates `README.md` previews, and generates a `docs/ui-gallery.md` gallery — all fully automated.

---

## 🚀 What It Does
- **Processes images** in the `docs/ui/` folder of any repository.
- **Optimizes PNGs** (size, scaling, radius) for consistent UI previews.
- **Generates previews in README.md** (up to 4 thumbnails + "See More" link).
- **Creates/updates a gallery page** `docs/ui-gallery.md` with all UI screenshots.
- **Reusable GitHub Action:** other repos call this workflow with a single job.
- **Binary caching & artifacts:** speeds up Rust builds and reuses binaries across runs.
- **Resilient pushes:** retries with rebase/force-push fallback to avoid workflow failures.

---

## 🧩 Workflow Structure
- **`image-processor.yml`** (in `.github/workflows/`):  
  The reusable GitHub Action workflow.
- **Rust binary (`image-processor`)**:  
  The core processor built in Rust.
- **Caching system:**  
  Uses `actions/cache` + `upload-artifact`/`download-artifact` to avoid rebuilding unnecessarily.

---

## 📦 Inputs (Configurable by Caller)

| Input            | Type    | Default     | Description                                |
|------------------|---------|-------------|--------------------------------------------|
| `image_folder`   | string  | `docs/ui/`  | Path to images in the calling repo         |
| `readme_path`    | string  | `README.md` | Path to repo’s README file                 |
| `enable_gallery` | boolean | `true`      | Enable/disable UI gallery generation       |
| `max_width`      | number  | `300`       | Maximum image width in pixels              |
| `target_radius`  | number  | `6.5`       | Border radius percentage (e.g. 6.5%)       |
| `check_size`     | boolean | `true`      | Enforce size checks                        |
| `check_radius`   | boolean | `true`      | Enforce radius checks                      |
| `fast_check`     | boolean | `true`      | Enables faster validation mode             |
| `columns`        | number  | `2`         | Columns for preview/gallery (1 or 2)       |

---

## 🔑 Required Secrets
- `IMAGE_PROCESS_PAT` → **GitHub token** with `repo` permissions 
- Used for checking out code, pushing changes and caching binaries.   
**Note:** token is needed only if repository is private.

---

## 📌 Example Usage of Workflow in Another Repository

```yaml
name: Process Images

on:
  push:
    paths:
      - 'docs/ui/**.png'
  pull_request:
    paths:
      - 'docs/ui/**.png'
  workflow_dispatch:

jobs:
  process-images:
    name: Process UI Images
    uses: Falcon12120/automation-workflows/.github/rust-ui-processor/workflows/ui-processor.yml@main
    with:
      image_folder: docs/ui/
      readme_path: README.md
      enable_gallery: true
      check_size: true
      max_width: 290
      check_radius: true
      target_radius: 6.5
      fast_check: true
      columns: 2
    secrets:
      IMAGE_PROCESS_PAT: ${{ secrets.IMAGE_PROCESS_PAT }}
````

---

## ⚡ Highlights

* Written in Rust → performance, safety, reliability.
* Reusable workflow → plug-and-play across multiple repos.
* Handles binary caching, retries and resilient pushes.
* Demonstrates **CI/CD, DevOps and workflow automation skills**.

---

## 🧑‍💻 Notes

This module shows:

* **System-level Rust development** (not just scripting).
* **GitHub Actions expertise** (cache, artifacts, reusable workflows).
* **DevOps mindset** (build once, reuse everywhere).
* Ability to build **scalable automation tools** for an engineering org.

---