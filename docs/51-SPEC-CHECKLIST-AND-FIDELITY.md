# 51 — Spec checklist and how fidelity is measured

Fidelity is the most critical property of this product. It is measured, per construct, against
something other than our own code.

## The checklist

[spec/](spec/README.md) lists **every tag, property and enum value**: 8730 rows from 77 schema files, each
assigned to an area, a tier and a phase. `tools/spec-checklist/generate.py` writes it, following the idea of
OpenCalc's `tools/fidelity-audit`: the "what should exist" side comes from the standard, because every
construct OpenCalc missed was missed by a checklist written from memory.

| Rows | Count | | Rows | Count |
| --- | ---: | | --- | ---: |
| Elements in a type | 2371 | | Top-level elements | 301 |
| Enum values | 2256 | | Simple types | 499 |
| Attributes | 1740 | | Group and attribute-group uses | 245 |
| Types and groups | 1208 | | Global attributes | 74 |
| Wildcards (`xsd:any`) | 25 | | Type extensions | 11 |

**Checked against the raw schemas by separate counting:** types, simple types, elements, attributes and enum
values all match. That check caught 74 missing global attributes (including `r:id` and `r:embed`) in the first
version of the generator, so it should run whenever the generator changes. It is manual until `SPEC-03`.

**Gates:** an unassigned type or namespace fails generation (seen failing), and `--check` fails on stale
output (seen failing). Status columns survive regeneration.

**Next, as in OpenCalc:** an audit tool reads the importer and exporter and fills the columns from the code,
so a claim of coverage cannot drift from it (`SPEC-01`).

## What the checklist cannot tell you

**A schema says what is legal. It does not say what it means.** A complete checklist is necessary and not
sufficient. Most real fidelity bugs live in behaviour, so behaviours have their own rows.

| ID | Behaviour, each with its own fixture set | Tier |
| --- | --- | --- |
| BEH-01 | Placeholder matching between slide, layout and master, by index and type | 1D |
| BEH-02 | Property inheritance order for each property family | 1D |
| BEH-03 | Colour transform order, HSL and gamma arithmetic | 1C |
| BEH-04 | Colour map and override resolution; theme override parts | 1C |
| BEH-05 | Group transform composition: nested scale, rotation, flips | 2B |
| BEH-06 | Text in rotated and flipped shapes; `upright` and `rot` on body | 2D |
| BEH-07 | Geometry guide evaluation, text rectangle, connection sites, for all 187 | 2B |
| BEH-08 | Line breaking: Latin, CJK rules and kinsoku, hanging punctuation, RTL, mixed runs | 2D |
| BEH-09 | Line spacing: percent against points, first-line, font-metric sources | 2D |
| BEH-10 | Autofit: stored scale, recomputation, `spAutoFit` resize, empty paragraphs | 2D |
| BEH-11 | Bullets: inheritance, autonumber restarts and continuation, picture bullets | 2D |
| BEH-12 | Tabs, columns, vertical text, anchoring, insets, overflow | 2D |
| BEH-13 | Font fallback and substitution by script; theme font and `+mj-lt` references | 2D |
| BEH-14 | Tables: style part evaluation, row growth, merges, borders, cell margins | 2E |
| BEH-15 | Picture crop, tile, stretch, `srcRect` beyond bounds, recolour effects | 2E |
| BEH-16 | Gradient interpolation, path gradients, pattern scaling, fill rotating with shape | 2B |
| BEH-17 | Shadow, glow, soft edge and reflection compositing order | 2C |
| BEH-18 | `mc:AlternateContent` choice among supported namespaces | 1A |
| BEH-19 | Fields: slide number, date formats, footers from layout | 2D |
| BEH-20 | Hidden slides, background inheritance, `showMasterSp`, master shape visibility | 2A |
| BEH-21 | Default values: an absent attribute, against the schema default, against PowerPoint's own | every tier |
| BEH-22 | Rounding: EMU to pixel, centipoint, angle precision on round-trip | 1D, 2 |
| BEH-23 | Timing node semantics, restart, `fill`, sequence navigation | 8 |
| BEH-24 | Round-trip: attribute order, prefixes, namespace declarations, `Ignorable` preserved on untouched elements | 1A |
| BEH-25 | Stale derived content: SmartArt drawing, `mc:Fallback`, chart caches after an edit ([52](52-COMPETITOR-STUDY.md)) | 1N |
| BEH-26 | Retention of 2016+ charts, when not rendered | 1J |
| BEH-27 | 3D and effects shown or retained, never flattened on save | 1E, 2C |
| BEH-28 | WordArt and text effects: transforms, gradient text, transparency, outline | 1F, 2D6 |
| BEH-29 | Malformed but real input: `p:pos` with no `p:cm`, unknown table style GUID. Never crashes | every tier |

Further rows are added whenever a fixture finds one. Before each tier, read the matching part of
[MS-OI29500](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/1fd4a662-8623-49c0-82f0-18fa91b413b8)
(`SPEC-02`): Microsoft's own list of where Office departs from the standard.

## Fidelity ladder

Measured separately, since passing one says nothing of the next.

| Level | Question | How | Blocked on |
| --- | --- | --- | --- |
| F1 Bytes | Does an unedited save match? | Byte compare | — |
| F2 Parts | Do untouched parts match after an edit? | Per-part hash, with a per-edit allow-list | — |
| F3 Semantics | Is import → export → import a fixed point? | Model compare | — |
| F4 Validity | Is the XML schema-valid? | `xmllint` against vendored XSD | — |
| F5 Acceptance | Does PowerPoint open it with no repair prompt? | **PowerPoint itself**, by automation or by hand | `PV-01` |
| F6 Pixels | Does it look like PowerPoint's rendering? | Perceptual compare against **PowerPoint's** PNG or PDF | `PV-01` |
| F7 Second opinion | Does LibreOffice agree, where it differs? | Convert and compare; a difference is a question, not a verdict | Off on desktops (`FID-04`) |
| F8 Edit | Does an edit look right in PowerPoint? | Scripted edit, reopen, compare with a PowerPoint-made equivalent | `PV-01` |

**Thresholds are per fixture and set from evidence.** A global "95% similar" would hide a missing bullet, so
comparison also checks text positions, per-region differences and structure from PDF output.

**Reality check** (`DEC-04`). Until `PV-01`, only F1 to F4 and F7 run, plus Microsoft's validator (`F-006`) and real PowerPoint-written decks as input (`F-007`). Those prove we keep the data and write valid
files. They cannot prove we match PowerPoint, and no claim saying so may be made from them.

**The gallery's agreement score** (`FH-01`, `tools/fidelity/score.py`, run by `pages.yml` on
every gallery build). Each of our slide PNGs is compared with LibreOffice's at 96 dpi: the
share of pixels whose colour is more than delta-E 10 from every pixel of the other image within
1 px, taken both ways so missing content counts (R1's measure in
[06](06-ROADMAP-AND-DELIVERY.md)), and SSIM on luma. `fidelity.json` holds it per slide, per
deck and overall, with how many slides are within R1's 1% and within 5%. It is F7, a second
opinion: it measures **agreement with LibreOffice**, labelled with LibreOffice's version, and
says nothing about PowerPoint. Where we differ, either may be wrong. The runner's fonts are not
the deck's, so font substitution dominates slides with small text, in both renders. A deck
whose two renders have different slide counts (hidden slides, a failed render) is not scored,
and says so. It is recorded, not a gate.

**Measured so far** (mean share of pixels differing, over the gallery's 111 scored slides, read
from each run's `fidelity.json`; agreement with LibreOffice, never a PowerPoint measure):

| Run | At | LibreOffice | Mean differ (agree) | Median | SSIM mean | Within 5% | Within 1% |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| 42492516147, first score (`FH-03`) | 2026-11-08 | as recorded in the run | 7.7% (92.3%) | — | 0.83 | — | — |
| 42496976677 (`FH-05`) | 2026-11-08 | as recorded in the run | 6.7% (93.3%) | — | — | — | — |
| 42511686894 | `1ba6443` | 26.2.6.3 | **5.4% (94.6%)** | 4.4% | 0.869 | 64 | 14 |

After `FH-06`'s three fixes, a local `render-deck` of the same decks scored against run
42512837899's LibreOffice images gives 5.0% mean (95.0% agreement), median 3.7%, SSIM 0.874,
70 within 5% and 14 within 1%: a local measure on the same references, not a CI run, until the
next `pages` run records it.

The gains came from fixes in our renderer (`FH-03`, `FH-05`, `FH-04`'s done items), from the
harness cropping rather than resampling the reference, and from giving the runner's LibreOffice
Carlito, Caladea and Noto CJK, so part of the change is the reference, not us. The latest numbers
are on the [gallery](https://casualoffice.github.io/openpresent/gallery/), labelled with the
LibreOffice version and commit.

## Corpus

| Set | Source | Purpose |
| --- | --- | --- |
| `generated/` | Deterministic script | One construct per deck, one per checklist area; every preset, every enum value |
| `hostile/` | Script | Every `SEC-*` threat with a fixture |
| `references/` | PowerPoint-exported PNG and PDF, by version | The pixel target |
| `producers/` | Decks from PowerPoint Win and Mac, Keynote, Google Slides, LibreOffice, python-pptx | Different writers make different valid files |
| `real/` | Rights-reviewed real decks | The long tail, since generated decks only contain what we thought of |

Everything is checksummed in a manifest, and third-party material needs a rights review before commit. Confidential
decks are never committed.

**What a mark rests on (`CLOSE-02`).** `y` needs code, a test and a fixture, and two tests then check that behaviour agrees with the marks, by local name: `casual-slide-import/tests/checklist_rows.rs` removes a construct from a deck and the model must change; `casual-slide-export/tests/checklist_rows.rs` removes it and asks for the deck as it was, and the save must put it back. `r` means the node stays in the tree through any edit and the report names it, resting on `casual-slide-export/tests/retention.rs` and `casual-slide-import/tests/report.rs`, which puts a stranger on each element and fails if the report does not change. `n/a` is allowed only where the audit can see it from the schemas. A blank in a closing tier needs a line in [spec/open-rows.tsv](spec/open-rows.tsv) naming a tracker row and the reason, or `audit.py --check` fails.

**Coverage is to be a gate (`SPEC-01`):** the audit reports which checklist rows no fixture exercises. A row with no fixture cannot be `y`.
