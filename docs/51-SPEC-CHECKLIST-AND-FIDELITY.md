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
| F7 Second opinion | Does LibreOffice agree, where it differs? | Convert and compare; a difference is a question, not a verdict | Available |
| F8 Edit | Does an edit look right in PowerPoint? | Scripted edit, reopen, compare with a PowerPoint-made equivalent | `PV-01` |

**Thresholds are per fixture and set from evidence.** A global "95% similar" would hide a missing bullet, so
comparison also checks text positions, per-region differences and structure from PDF output.

**Reality check** (`DEC-04`). Until `PV-01`, only F1 to F4 and F7 run, plus Microsoft's validator (`F-006`) and real PowerPoint-written decks as input (`F-007`). Those prove we keep the data and write valid
files. They cannot prove we match PowerPoint, and no claim saying so may be made from them.

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

**Coverage is to be a gate (`SPEC-01`):** the audit reports which checklist rows no fixture exercises. A row with no fixture cannot be `y`.
