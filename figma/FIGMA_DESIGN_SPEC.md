# RusticPlayer • Figma Design Specification & Transformation Guide

> **Workflow**: Design in Figma first $\rightarrow$ Inspect & Tweak $\rightarrow$ Transform to GPUI Rust code.

---

## 1. Overview & Ready-to-Use Figma Assets

All assets are located in `f:/VSCodium/Github/RusticPlayer/figma`:

| File | Purpose | Canvas Dimensions |
| :--- | :--- | :--- |
| [`rustic_player_full_ui.svg`](file:///f:/VSCodium/Github/RusticPlayer/figma/rustic_player_full_ui.svg) | **Full Application Mockup**: Complete desktop player with Titlebar, Sidebar, Hero Banner, Album Grid, and the **Floating Glass Island Dock** pinned at the bottom over scrolling content. | 1440 × 960 px |
| [`floating_glass_dock_variants.svg`](file:///f:/VSCodium/Github/RusticPlayer/figma/floating_glass_dock_variants.svg) | **4 Distinct Dock Concepts**: VisionOS Frosted, Cyberpunk Neon Prism, Smoked Obsidian, and Luminescent Ice. | 1200 × 1100 px |
| [`rustic_player_design_tokens.svg`](file:///f:/VSCodium/Github/RusticPlayer/figma/rustic_player_design_tokens.svg) | **Design System Tokens**: Color swatches, Glassmorphism 4-layer optical formula, Geometry/radii, and Typography scale. | 1200 × 920 px |
| [`preview.html`](file:///f:/VSCodium/Github/RusticPlayer/figma/preview.html) | **Interactive Webview**: Inspect all vectors in high DPI with zoom and 1-click clipboard copy. | Full Screen |
| [`copy_to_clipboard.ps1`](file:///f:/VSCodium/Github/RusticPlayer/figma/copy_to_clipboard.ps1) | **PowerShell Quick Script**: Copies any vector directly into Windows Clipboard for instant paste into Figma. | CLI |

---

## 2. Instant Import into Figma (3 Methods)

### Method A: Paste from Clipboard (Fastest — Already Copied!)
1. We already executed `copy_to_clipboard.ps1 -Target full`. The full UI SVG data is currently sitting on your Windows clipboard!
2. Switch to your open **Figma** desktop app.
3. Open any canvas and press <kbd>Ctrl</kbd> + <kbd>V</kbd>.
4. Figma will immediately generate a clean Frame titled `rustic_player_full_ui` with editable layers, text, gradients, and strokes!

*To switch targets anytime in PowerShell:*
```powershell
.\figma\copy_to_clipboard.ps1 -Target full      # Full 1440x960 desktop app
.\figma\copy_to_clipboard.ps1 -Target variants  # 4 Floating Dock concepts
.\figma\copy_to_clipboard.ps1 -Target tokens    # Design system tokens
```

### Method B: Drag and Drop
- Drag any `.svg` file from `f:\VSCodium\Github\RusticPlayer\figma\` directly into the Figma canvas window.

### Method C: Via `preview.html`
- Open `f:\VSCodium\Github\RusticPlayer\figma\preview.html` in your browser, switch tabs, and click **"Copy SVG to Clipboard"**.

---

## 3. Optical Glass Physics: Why It Looks Like Real Glass

In computer graphics, a dark rectangle with 85% opacity looks like dull dark plastic because:
1. **There is no content behind it to refract/show through.**
2. **Real physical glass has specular reflection along its top curved bevel.**
3. **Real glass has a soft elevation drop shadow that physically separates it from the surface below.**

### The 4-Layer Glass Formula:
```
┌────────────────────────────────────────────────────────┐
│ Layer 4: Convex Lens Sheen (Top 34px, #FFF 14% -> 0%)  │
├────────────────────────────────────────────────────────┤
│ Layer 3: Specular Hairline Rim (1.2px, #FFF 65% -> 10%)│
├────────────────────────────────────────────────────────┤
│ Layer 2: Translucent Tint Body (#140D26 @ 42% Opacity) │
├────────────────────────────────────────────────────────┤
│ Layer 1: Ambient Drop Shadow (Y: 16px, Blur: 36px)     │
└────────────────────────────────────────────────────────┘
                 ▲ Content Passes Underneath ▲
```

---

## 4. Figma Layer Structure & GPUI Rust Mapping

When you inspect or modify layers in Figma, here is how each layer maps directly into our Rust GPUI codebase (`crates/ui-kit/src/organisms/now_playing_bar.rs` and `crates/app/src/main.rs`):

| Figma Layer Name | Purpose & Values | GPUI Rust Code Equivalent |
| :--- | :--- | :--- |
| `Floating-Glass-Dock` | Container frame, `1150 x 76`, `rx: 22` | `div().absolute().bottom(px(16.0)).left_1_2().-translate_x_1_2().h(px(76.0)).rounded(px(22.0))` |
| `Dock-Drop-Shadow` | Drop Shadow: `0 16px 36px rgba(0,0,0,0.65)` | `.shadow(Elevation::lg())` |
| `Dock-Glass-Fill` | Translucent linear gradient `rgba(36,24,62,0.52)` to `rgba(14,8,27,0.68)` | `.bg(rgba(0x140D266B))` |
| `Dock-Specular-Rim` | 1.2px stroke (`#FFF` 65% $\rightarrow$ 10%) | `.border_1().border_color(rgba(0xFFFFFF3E))` |
| `Dock-Convex-Sheen` | Top 34px highlight (`#FFF` 14% $\rightarrow$ 0%) | `div().absolute().top_0().left_0().right_0().h(px(32.0)).bg(rgba(0xFFFFFF0F))` |
| `Dock-Center-Dynamic-Island` | Recessed capsule `550 x 60`, `rx: 16`, fill `#05030A` @ 55% | `div().w(px(540.0)).h(px(58.0)).rounded(px(16.0)).bg(rgba(0x06030F66))` |
| `Badge-Glass-Toggle` | Pill `84 x 34`, `rx: 17`, Cyan glow | `div().px_3().py_1().rounded_full().border_1().border_color(rgba(0x38BDF866))` |

---

## 5. Iteration & Transformation Workflow

1. **Inspect in Figma**:
   - Check the margins, corner radius (`22px` vs `26px` vs `pill`), height (`76px`), and button alignments.
   - Test color variations (Cyan/Purple accent vs Amber/Orange vs Monochromatic).
   - Adjust the fill opacity (40% vs 50% vs 60%) to your personal taste.
2. **Tell Antigravity**:
   - Tell me what changes you made or which variant you like best (e.g. *"I want variant A with slightly more rounded corners and cyan accent"* or *"I tweaked the dock width to 1100px and added 18px radius"*).
3. **Automated Transformation**:
   - I will immediately translate the exact Figma properties into `crates/ui-kit` and `crates/app`, build the Rust binary with zero warnings, and verify it live on screen with high-FPS smooth playback!
