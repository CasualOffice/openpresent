# Bundled fonts

**In this repository:** copied unmodified from OpenCalc's `crates/casual-calc-text/fonts/`
at `b23c67c` (see `../PROVENANCE.md`), embedded by `casual-slide-text`, and listed in the
repository's `NOTICE`. The text below is OpenCalc's, which in turn kept OpenDoc's; the
crate and document names it gives are theirs. The font set is `DEC-16`'s, under ADR-016.

**Added here, not from OpenCalc (2026-11-09, `T2D1-04`):** two Arabic families in
`noto/`, because Arabic text drew nothing. Unhinted static faces from Google's official
releases, unmodified; the OFL text, identical in both releases, is
[`LICENSES/OFL-1.1-NotoArabic.txt`](LICENSES/OFL-1.1-NotoArabic.txt).

| File | Release | SHA-256 |
| --- | --- | --- |
| `noto/NotoSansArabic-Regular.ttf` | `NotoSansArabic-v2.013`, https://github.com/notofonts/arabic/releases/download/NotoSansArabic-v2.013/NotoSansArabic-v2.013.zip (zip SHA-256 `1301aceaea84c501cf2e6dcfb3182e2328c8eae5725817fcb239672bda7154f1`), `NotoSansArabic/unhinted/ttf/` | `bd86ca02f087d7f3c3788ba458fb6b73744c7639ed276b8d870dba6def6c40d0` |
| `noto/NotoSansArabic-Bold.ttf` | same | `f4cb79f842a07d1bcf597879574358fb7010f73761396d1d0f2445dfedb4d2ca` |
| `noto/NotoNaskhArabic-Regular.ttf` | `NotoNaskhArabic-v2.021`, https://github.com/notofonts/arabic/releases/download/NotoNaskhArabic-v2.021/NotoNaskhArabic-v2.021.zip (zip SHA-256 `6c050ab9bd087d69b733c505a7576e60c528c2f33cd7b91005a5bd7da4514032`), `NotoNaskhArabic/unhinted/ttf/` | `c34fdbd98af4dbc45ca192a23d2eeb77032add83086f5fe32957d23b3f36b221` |
| `noto/NotoNaskhArabic-Bold.ttf` | same | `380ac34b249d336e1a43c21053df440594e1e3db8f75fabd4d85b649ca1ea4c6` |

Roles: fallbacks only, last in the coverage chain, never a substitute for a requested
name. A run's Arabic is drawn in Noto Naskh Arabic when the run's family is a serif one
(Times New Roman, the Office theme's Arabic font), else in Noto Sans Arabic (Arial,
Tahoma). Italic draws upright.

Fonts embedded into `casual-doc-layout` for deterministic, WASM-safe text layout
(the shaper registers these into an empty font collection — no system-font
discovery — so layout metrics are reproducible on every host; see
`docs/43-PHASE-1C-LAYOUT-RENDERING-DESIGN.md` §1).

| File | Family | License | Source |
| --- | --- | --- | --- |
| `Roboto-Regular.ttf` | Roboto | Apache-2.0 | [github.com/googlefonts/roboto-2](https://github.com/googlefonts/roboto-2) |
| `Roboto-Bold.ttf` | Roboto Bold | Apache-2.0 | same |
| `Roboto-Italic.ttf` | Roboto Italic | Apache-2.0 | same |
| `Roboto-BoldItalic.ttf` | Roboto Bold Italic | Apache-2.0 | same |
| `Caladea-Regular.ttf` | Caladea | Apache-2.0 | [github.com/huertatipografica/Caladea](https://github.com/huertatipografica/Caladea) (Huerta Tipografía; croscore) |
| `Caladea-Bold.ttf` | Caladea Bold | Apache-2.0 | same |
| `Caladea-Italic.ttf` | Caladea Italic | Apache-2.0 | same |
| `Caladea-BoldItalic.ttf` | Caladea Bold Italic | Apache-2.0 | same |
| `Carlito-Regular.ttf` | Carlito | SIL OFL-1.1 | [github.com/googlefonts/carlito](https://github.com/googlefonts/carlito) (The Carlito Project Authors; croscore) |
| `Carlito-Bold.ttf` | Carlito Bold | SIL OFL-1.1 | same |
| `Carlito-Italic.ttf` | Carlito Italic | SIL OFL-1.1 | same |
| `Carlito-BoldItalic.ttf` | Carlito Bold Italic | SIL OFL-1.1 | same |
| `liberation/LiberationSans-Regular.ttf` | Liberation Sans | SIL OFL-1.1 | [github.com/liberationfonts/liberation-fonts](https://github.com/liberationfonts/liberation-fonts) (Red Hat / Google; `liberation-fonts-ttf-2.1.5`) |
| `liberation/LiberationSans-Bold.ttf` | Liberation Sans Bold | SIL OFL-1.1 | same |
| `liberation/LiberationSans-Italic.ttf` | Liberation Sans Italic | SIL OFL-1.1 | same |
| `liberation/LiberationSans-BoldItalic.ttf` | Liberation Sans Bold Italic | SIL OFL-1.1 | same |
| `liberation/LiberationSerif-Regular.ttf` | Liberation Serif | SIL OFL-1.1 | same |
| `liberation/LiberationSerif-Bold.ttf` | Liberation Serif Bold | SIL OFL-1.1 | same |
| `liberation/LiberationSerif-Italic.ttf` | Liberation Serif Italic | SIL OFL-1.1 | same |
| `liberation/LiberationSerif-BoldItalic.ttf` | Liberation Serif Bold Italic | SIL OFL-1.1 | same |
| `liberation/LiberationMono-Regular.ttf` | Liberation Mono | SIL OFL-1.1 | same |
| `liberation/LiberationMono-Bold.ttf` | Liberation Mono Bold | SIL OFL-1.1 | same |
| `liberation/LiberationMono-Italic.ttf` | Liberation Mono Italic | SIL OFL-1.1 | same |
| `liberation/LiberationMono-BoldItalic.ttf` | Liberation Mono Bold Italic | SIL OFL-1.1 | same |

Roboto and Caladea are licensed under the Apache License 2.0 — the same license
as this repository. Each carries the Apache-2.0 license string in its `name`
table (verified on import).

Carlito and the **Liberation** families are licensed under the **SIL Open Font
License 1.1** (each carries the OFL license record and URL, `name` ID 13/14,
`scripts.sil.org/OFL` — verified on import). The OFL is a permissive license that
governs only the font file, not this Apache-2.0 code, and carries **no** copyleft
or relicensing effect. The fonts are embedded as `include_bytes!` asset bytes,
not pulled in as a crate dependency, so `cargo-deny` does not scan them (no
`deny.toml` allowlist change is required). The OFL obligations are met by shipping
each license text unmodified alongside the fonts in
[`LICENSES/OFL-1.1-Carlito.txt`](LICENSES/OFL-1.1-Carlito.txt) and
[`LICENSES/OFL-1.1-Liberation.txt`](LICENSES/OFL-1.1-Liberation.txt); we
redistribute the fonts unmodified and do not sell them standalone or reuse a
reserved font name for a modified font.

Provenance: the Carlito TTFs and `OFL.txt` were downloaded from
`googlefonts/carlito` (`main`, commit
`3a810cab78ebd6e2e4eed42af9e8453c4f9b850a`) under `fonts/ttf/`. The Liberation
TTFs and `LICENSE` were extracted from the official
`liberation-fonts-ttf-2.1.5.tar.gz` release tarball
(`github.com/liberationfonts/liberation-fonts`); each `.ttf` was verified as a
valid TrueType (sfnt version `0x00010000`) before committing.

## Roles (font resolver, `P1C-002b`)

- **Roboto** is the default family and the ultimate fallback (`FontId(0)..=3`).
- **Caladea** (`FontId(4)..=7`) is metric-compatible with **Cambria** (matching
  advances, so line breaks are preserved); the resolver maps Cambria → Caladea.
- **Carlito** (`FontId(8)..=11`) is metric-compatible with **Calibri** (matching
  advances); the resolver maps Calibri → Carlito. Calibri is the most common Word
  font, so this preserves layout for the majority of real documents.
- **Liberation Sans / Serif / Mono** (`FontId(12)..=23`) are LibreOffice's own
  metric-compatible substitutes for **Arial/Helvetica**, **Times New Roman**, and
  **Courier New**. The resolver maps those requested families (and, by generic
  class, other missing fonts) to them, so a document whose fonts are not installed
  breaks lines and paginates the way LibreOffice does — the fix for page-count and
  line-breaking divergence. The mapping table lives in
  `src/font_substitution.rs`; every non-exact substitution is reported, never
  silently swapped.
