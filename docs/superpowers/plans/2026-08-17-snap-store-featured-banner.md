# Snap Store Featured Banner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Produce a professional Noor Notes Snap Store featured banner at 1920 × 640 pixels using the exact application icon, approved title and tagline, and an accurate current Light Mode product preview.

**Architecture:** Generate only a restrained, text-free raster foundation with the built-in image-generation tool. Build the brand block and product preview deterministically in a temporary HTML/CSS composition using the repository’s existing SVG icon and current Light Mode screenshot, then render that composition through headless Chrome to the final PNG. Remove every temporary source and retain only the validated banner asset.

**Tech Stack:** Built-in image generation, HTML5/CSS, headless Google Chrome, existing SVG and PNG project assets, POSIX shell

## Global Constraints

- Final asset: `data/store/noor-notes-featured-banner.png`.
- Exact canvas: 1920 × 640 pixels, 3:1 aspect ratio.
- Final format: RGB PNG below 2 MB.
- Exact product name: `Noor Notes`.
- Exact tagline: `Calm notes. Powerful editor.`
- Preserve `data/io.github.saamaamr.NoorNotes.svg` without redrawing or modifying it.
- Use `data/screenshots/noor-notes-library.png` as the accurate Light Mode product preview.
- Add no other marketing copy, badge, watermark, person, third-party logo, or personal data.
- Avoid glassmorphism, neon glow, particles, floating blobs, aggressive perspective, and visual clutter.
- Keep all important content inside generous safe margins and readable at 720 × 240.
- Do not modify application code, existing icons, existing screenshots, or the two untracked Snap packages.
- Do not push unless the user explicitly requests it.

---

### Task 1: Generate the restrained raster foundation

**Files:**
- Create temporarily: `tmp/banner-work/foundation.png`

**Interfaces:**
- Consumes: approved Graphite/navy, indigo, teal, and warm-yellow visual direction.
- Produces: a text-free raster foundation consumed by `tmp/banner-work/banner.html`.

- [ ] **Step 1: Create the isolated working directory**

Run:

```bash
mkdir -p tmp/banner-work data/store
```

Confirm `tmp/` is ignored before storing generated work:

```bash
git check-ignore -q tmp/banner-work
```

Expected: exit code 0. If `tmp/` is not ignored, keep the work under `/tmp/noor-notes-banner-work` instead and use absolute `file:///home/mamun/Documents/noor-notes/...` URLs for the icon and screenshot in the temporary HTML; do not change `.gitignore` for a temporary asset.

- [ ] **Step 2: Generate one background foundation with the built-in image tool**

Use this exact structured prompt:

```text
Use case: ads-marketing
Asset type: Snap Store featured banner background foundation
Primary request: Create a restrained abstract background for a calm, professional Linux note-taking application. This is background atmosphere only; all branding and interface content will be composited later.
Scene/backdrop: deep Graphite and navy field with one very soft indigo area and one subtle teal area, generous quiet negative space, slightly brighter toward the right-side product area
Style/medium: refined editorial technology branding, smooth tonal depth, minimal and professional
Composition/framing: very wide 3:1 landscape composition, visually quiet left half, gentle depth on the right, no central subject
Lighting/mood: calm, trustworthy, focused, low-contrast ambient light
Color palette: #1c1d20, #111827, restrained #4f6fe8 and muted teal, tiny warm-yellow influence only if subtle
Constraints: background only; no text; no letters; no logos; no icons; no application windows; no screenshots; no people; no objects; no watermark; no hard edges; no high-frequency texture
Avoid: glassmorphism, neon, particles, floating blobs, strong multicolour gradients, lens flare, 3D objects, busy patterns
```

Use the generated image only as a supporting foundation, not as the final banner.

- [ ] **Step 3: Save and inspect the selected foundation**

Copy the selected built-in result from the returned `$CODEX_HOME/generated_images/...` path to:

```text
tmp/banner-work/foundation.png
```

Inspect it at original resolution. Reject and regenerate once if it contains text-like marks, logos, objects, strong neon, or busy texture. The accepted foundation must remain subordinate to the deterministic content.

---

### Task 2: Compose the exact icon, copy, and product preview

**Files:**
- Create temporarily: `tmp/banner-work/banner.html`
- Create: `data/store/noor-notes-featured-banner.png`

**Interfaces:**
- Consumes: `tmp/banner-work/foundation.png`, `data/io.github.saamaamr.NoorNotes.svg`, and `data/screenshots/noor-notes-library.png`.
- Produces: the final 1920 × 640 banner PNG.

- [ ] **Step 1: Create the deterministic HTML composition**

Create `tmp/banner-work/banner.html` with this complete content, preserving the exact text:

```html
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=1920, initial-scale=1">
  <title>Noor Notes featured banner</title>
  <style>
    * { box-sizing: border-box; }
    html, body {
      width: 1920px;
      height: 640px;
      margin: 0;
      overflow: hidden;
      background: #111827;
      font-family: system-ui, "Cantarell", "Noto Sans", sans-serif;
    }
    .banner {
      position: relative;
      width: 1920px;
      height: 640px;
      overflow: hidden;
      color: #f8fafc;
      background:
        radial-gradient(circle at 68% 22%, rgba(79,111,232,.20), transparent 37%),
        radial-gradient(circle at 88% 84%, rgba(39,130,132,.15), transparent 34%),
        linear-gradient(115deg, #111827 0%, #171b29 58%, #151b24 100%);
    }
    .foundation {
      position: absolute;
      inset: -30px;
      width: calc(100% + 60px);
      height: calc(100% + 60px);
      object-fit: cover;
      filter: blur(28px) saturate(.72) contrast(.92);
      opacity: .24;
    }
    .quieting-layer {
      position: absolute;
      inset: 0;
      background: linear-gradient(90deg, rgba(10,15,27,.74) 0%, rgba(10,15,27,.40) 43%, rgba(10,15,27,.10) 100%);
    }
    .brand {
      position: absolute;
      left: 124px;
      top: 118px;
      width: 650px;
      z-index: 2;
    }
    .icon {
      display: block;
      width: 150px;
      height: 150px;
      margin-bottom: 30px;
      filter: drop-shadow(0 14px 28px rgba(0,0,0,.24));
    }
    h1 {
      margin: 0;
      color: #ffffff;
      font-size: 76px;
      font-weight: 750;
      line-height: 1;
      letter-spacing: -2.5px;
    }
    .tagline {
      margin: 22px 0 0;
      color: #cbd5e1;
      font-size: 31px;
      font-weight: 470;
      line-height: 1.25;
      letter-spacing: -.3px;
    }
    .preview-accent {
      position: absolute;
      left: 848px;
      top: 86px;
      width: 934px;
      height: 492px;
      border: 1px solid rgba(125,211,252,.14);
      border-radius: 29px;
      background: rgba(79,111,232,.13);
      transform: translate(22px, 16px);
    }
    .preview {
      position: absolute;
      margin: 0;
      left: 822px;
      top: 62px;
      width: 970px;
      height: 516px;
      overflow: hidden;
      border: 1px solid rgba(226,232,240,.28);
      border-radius: 26px;
      background: #ffffff;
      box-shadow: 0 30px 70px rgba(0,0,0,.34), 0 3px 12px rgba(0,0,0,.18);
    }
    .preview img {
      display: block;
      width: 100%;
      height: 100%;
      object-fit: cover;
      object-position: center top;
    }
    .edge-line {
      position: absolute;
      left: 124px;
      bottom: 54px;
      width: 82px;
      height: 4px;
      border-radius: 2px;
      background: linear-gradient(90deg, #e7bf4d, #4f6fe8);
      opacity: .88;
    }
  </style>
</head>
<body>
  <main class="banner" aria-label="Noor Notes featured banner">
    <img class="foundation" src="foundation.png" alt="">
    <div class="quieting-layer"></div>
    <section class="brand">
      <img class="icon" src="../../data/io.github.saamaamr.NoorNotes.svg" alt="">
      <h1>Noor Notes</h1>
      <p class="tagline">Calm notes. Powerful editor.</p>
    </section>
    <div class="preview-accent"></div>
    <figure class="preview">
      <img src="../../data/screenshots/noor-notes-library.png" alt="">
    </figure>
    <div class="edge-line"></div>
  </main>
</body>
</html>
```

- [ ] **Step 2: Render the final asset through headless Chrome**

Run:

```bash
google-chrome --headless=new --disable-gpu --hide-scrollbars --allow-file-access-from-files --window-size=1920,640 --screenshot=/home/mamun/Documents/noor-notes/data/store/noor-notes-featured-banner.png file:///home/mamun/Documents/noor-notes/tmp/banner-work/banner.html
```

Expected: Chrome reports a successfully written PNG and exits 0.

- [ ] **Step 3: Verify the hard banner contracts**

Run:

```bash
file data/store/noor-notes-featured-banner.png
test "$(stat -c %s data/store/noor-notes-featured-banner.png)" -lt 2097152
```

Expected: `PNG image data, 1920 x 640, 8-bit/color RGB` and file-size check exit code 0. If Chrome emits RGBA, re-render after adding `background-color: #111827` to both `html` and `body`; do not accept an alpha canvas.

---

### Task 3: Review store-scale readability and clean the workspace

**Files:**
- Create temporarily: `tmp/banner-work/thumbnail.html`
- Verify: `data/store/noor-notes-featured-banner.png`
- Remove: `tmp/banner-work/`

- [ ] **Step 1: Inspect the final banner at original resolution**

Open `data/store/noor-notes-featured-banner.png` with original detail and verify:

- the SVG icon is accurate and not distorted;
- `Noor Notes` and `Calm notes. Powerful editor.` are exact and fully visible;
- the Light Mode preview is crisp enough to communicate the product without competing with the brand block;
- the background contains no generated text, logo, watermark, object, or distracting texture;
- no content touches the canvas edge or appears clipped.

- [ ] **Step 2: Create and inspect a 720 × 240 store-card preview**

Create `tmp/banner-work/thumbnail.html`:

```html
<!doctype html>
<html><head><style>
html,body{margin:0;width:720px;height:240px;overflow:hidden;background:#111827}
img{display:block;width:720px;height:240px}
</style></head><body>
<img src="../../data/store/noor-notes-featured-banner.png" alt="Noor Notes banner preview">
</body></html>
```

Render it:

```bash
google-chrome --headless=new --disable-gpu --hide-scrollbars --allow-file-access-from-files --window-size=720,240 --screenshot=/tmp/noor-notes-banner-thumbnail.png file:///home/mamun/Documents/noor-notes/tmp/banner-work/thumbnail.html
```

Inspect `/tmp/noor-notes-banner-thumbnail.png`. The title and tagline must remain readable and the icon and preview must still be identifiable. If not, adjust only brand font size, preview scale, or horizontal spacing in `banner.html`, re-render the final banner, and repeat both resolution checks.

- [ ] **Step 3: Remove temporary composition files**

Delete the temporary HTML, generated foundation, and store preview through a scoped cleanup:

```bash
rm -rf /home/mamun/Documents/noor-notes/tmp/banner-work
rm -f /tmp/noor-notes-banner-thumbnail.png
```

Confirm:

```bash
test ! -e tmp/banner-work
test ! -e /tmp/noor-notes-banner-thumbnail.png
```

- [ ] **Step 4: Inspect the final repository scope**

Run:

```bash
git diff --check
git status --short
```

Expected: only `data/store/noor-notes-featured-banner.png` and this implementation plan are new tracked-work candidates; the two pre-existing `.snap` files remain untouched and untracked.

- [ ] **Step 5: Commit the final banner and plan**

Run:

```bash
git add data/store/noor-notes-featured-banner.png docs/superpowers/plans/2026-08-17-snap-store-featured-banner.md
git commit -m "docs: add Snap Store featured banner"
```

Do not push without a new explicit user request.
