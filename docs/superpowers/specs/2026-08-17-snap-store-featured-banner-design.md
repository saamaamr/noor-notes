# Snap Store Featured Banner Design

## Goal

Create a professional featured banner for the Noor Notes Snap Store listing. The banner should communicate the product identity—calm note-taking with a capable editor—while remaining clear at store-card scale and consistent with the application’s current Light and Graphite interface.

## Store Requirements

The final asset will follow the Snap Store banner guidance:

- 3:1 aspect ratio;
- 1920 × 640 pixels, the recommended resolution;
- PNG format;
- file size below 2 MB;
- important content kept away from the extreme edges so store cropping does not remove the icon, title, tagline, or primary interface preview.

The project asset will be saved as `data/store/noor-notes-featured-banner.png`. This is a new file and will not replace an existing application icon or screenshot.

## Approved Direction

Use an editorial product-showcase composition rather than an icon-only or bright full-interface banner.

### Left-side brand block

- Preserve the existing Noor Notes icon accurately rather than redrawing or restyling it.
- Display the exact product name: `Noor Notes`.
- Display the exact tagline: `Calm notes. Powerful editor.`
- Use clear, modern sans-serif typography with strong title hierarchy and readable tagline contrast.

### Right-side product preview

- Show a simplified, polished Light Mode library/editor preview inspired by the real application.
- Communicate the three-pane note workflow through a compact sidebar, restrained colour rails, note cards, and a readable editor surface.
- Keep preview text abstract or minimal so it does not compete with the title or risk inaccurate product claims.
- Use one primary interface composition rather than several overlapping screenshots.

### Background and visual treatment

- Use a deep Graphite/navy foundation aligned with the current dark appearance.
- Add restrained indigo and teal depth through a soft radial treatment or subtle tonal variation.
- Let the warm yellow note icon provide the principal brand contrast.
- Use subtle borders and controlled shadows to separate the Light Mode preview from the background.
- Avoid glassmorphism, neon glow, large decorative gradients, particles, floating blobs, excessive perspective, and visual clutter.

## Composition and Safe Area

Use a balanced two-part composition:

- approximately 42% of the width for the icon, title, and tagline;
- approximately 58% for the interface preview;
- generous internal padding on every edge;
- title and tagline aligned as one stable block;
- interface preview fully contained within the canvas, with no important controls clipped.

The banner must remain legible when scaled down. The title is the first visual anchor, followed by the icon, tagline, and product preview. No other marketing copy or badges will be added.

## Production Method

Use a hybrid production workflow:

1. Generate restrained background depth and product-showcase atmosphere as a raster foundation.
2. Composite the existing Noor Notes icon, exact title, exact tagline, and simplified product preview deterministically so text and brand geometry remain accurate.
3. Export the final composite as a 1920 × 640 RGB PNG.

The generated foundation must not contain legible text, substitute logos, watermarks, unrelated objects, people, or third-party branding. The exact icon and text will come from project-controlled assets and deterministic layout, not from generated approximations.

## Validation

Before delivery:

- inspect the banner at full resolution and at a small store-card preview size;
- confirm the icon matches `data/io.github.saamaamr.NoorNotes.svg`;
- confirm the title and tagline match the approved wording exactly;
- confirm the PNG is 1920 × 640, RGB, and below 2 MB;
- confirm there is no clipped content, watermark, personal data, or unrelated branding;
- confirm the asset remains readable in both light and dark surrounding pages;
- inspect the final Git diff and avoid unrelated changes.
