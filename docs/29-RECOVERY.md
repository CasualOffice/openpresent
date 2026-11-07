# 29 — Recovery: open what others refuse

**Owner instruction 24 (ADR-015).** OpenPresent opens and edits damaged and malformed decks, including ones Microsoft refuses or fails to open. That is a product advantage. A refusal is now the last resort, and it is reserved for files where nothing can be safely read at all.

## The rule

**Hostile and malformed are different things.** A malformed file gets the best reading we can make. A hostile construct is *neutralized*, not obeyed, and the rest of the deck still opens.

| Before | Now |
|---|---|
| A fault refuses the deck | A fault is repaired or set aside, and the deck opens |
| Limits refuse the deck | A limit stops work on that part. The part is retained and named |
| Strict reading in one walker | Strict first; on failure, a recovering reader for that part |

Nothing below ever obeys what it recovers from: **no entity is expanded, no external reference is followed, nothing is executed, no allocation trusts a file's number.** Those controls in [07](07-SECURITY-THREAT-MODEL.md) stand. Their *outcome* changes from "refuse the deck" to "neutralize and carry on".

## Recovery by layer

| Layer | Damage | What we do |
|---|---|---|
| **Container** | Truncated or missing central directory, bad offsets, wrong sizes | Rebuild the entry list by scanning local headers, bounded |
| | Prepended or trailing bytes | Find the ZIP inside |
| | CRC mismatch | Keep the bytes, report |
| | Duplicate or case-colliding names | Keep every entry retained; read the one with content that parses, preferring the last; report the rest |
| | Unsafe names (`../x`, control characters) | Never used as paths, since nothing is extracted. Kept as opaque entries and reported |
| | Bomb by ratio or size | That part is not decompressed. It is retained by raw copy and named; the deck opens without it |
| **Package** | Missing or broken `[Content_Types].xml` | Rebuild from extensions, roots and relationships |
| | Missing or broken `.rels` | Rebuild from part names and content |
| | No main-part relationship, or a wrong type | Find `presentation.xml` by name or root element; else build a deck from whatever slides exist |
| | Dangling, cyclic or escaping targets | Ignore that link, report |
| **XML** | DOCTYPE, entities | The DTD is skipped and never interpreted. Unknown references stay as literal text |
| | Unclosed or mismatched tags, multiple roots | The recovering reader closes what it can |
| | Bad attributes, repeated attributes, undeclared prefixes, invalid characters | Kept readable; first value wins; report |
| | Wrong encoding declared, stray BOM | Detect, decode, report |
| | Part cannot be read at all | That part is set aside and retained. If it is a slide, it becomes an empty slide marked as damaged, so slide order survives |
| **Model** | Out-of-range values, unknown enums, missing required items, duplicate ids | Left out and reported, or given new internal ids. Inheritance falls back along its chain |
| | Missing layout, master or theme | Use the nearest available; synthesize a minimal one in memory, and only write it if the user edits |

## What still refuses

Only what cannot be read at all, each with a clear message and its own code:

- Not a ZIP and nothing recoverable inside: for example random bytes.
- Compound File formats until their tiers land: encrypted, IRM and legacy `.ppt`.
- The whole-file size limit, which protects the machine.

## Saving a recovered deck

**Fidelity still holds, and damage is never silently rewritten.**
- Opening never changes the file. An unedited save returns the original bytes, damage included.
- The user is told the deck was repaired, and what was affected, in plain language.
- An edit, or an explicit repair, writes a **valid** file: the writer's preflight applies to the output, so we output clean decks even from damaged input.
- A part we could only partly read is rewritten only if the user edits it. Otherwise its bytes are kept.

## Why this is safe

Recovering parsers are a classic attack surface, so:
- One recovering reader per format, bounded in time, depth and memory like the strict one, with its own fuzz target.
- Every hostile fixture stays, and its expectation changes from "refused" to "opens, the attack did nothing": no expansion, no fetch, no text from it in a message.
- **Scanner differential (`SEC-P05`).** Two readers can still disagree about a duplicate entry. We cannot prevent that for other software, so we report it, and on save we write only one.
- A host can set `strict: true` to refuse instead, for deployments that prefer rejection to recovery, such as mail gateways. Default is recovery.

## How it is built

**One clean image.** The whole design is that readers and the writer never meet damage. `PresentationPackage::open` runs strict admission first. On any fault it repairs the package into a clean ZIP in memory that strict admission then accepts, and everything above reads that. The bytes as they arrived are kept apart, so an unedited save returns them, and a part made in the repair exists only until an edit writes it. A healthy deck takes the old path, and a part found damaged only when it is read starts the same repair then (`read_xml_part`). Before a save writes anything, every stream and XML part is seen to be sound (`prepare_for_edit`), so damage no reader reached is not carried into a new file; this is the cost of a first save, and it reads media. `open_strict` is the old refusal, with the old codes.

| Layer | Where | What it does |
|---|---|---|
| Container | `casual-slide-package/src/salvage.rs` | Its own bounded reader, since `zip` merges duplicate names and refuses damage. Directory if whole, local headers otherwise, forward only. Deflate through `flate2`, on one budget |
| Package and model | `casual-slide-ooxml/src/repair.rs` | Content types, relationships, the main part, and the chain from slide to layout to master to theme |
| XML | `casual-slide-ooxml/src/recover_xml.rs` | One loop, no recursion, and its output is what `walk` admits. `XmlTree::parse_recovering` keeps a tree byte-faithful: untouched, it writes back what arrived |
| Report | `casual-slide-ooxml/src/recovery.rs` | `recovery_report()`, codes `CS-REC-*` in [20](20-ERROR-CODE-REGISTRY.md), and sentences with no document text |

**A damaged part's bytes are never lost.** They go to a retained entry, `recovered/NNNNN-name.retained`, declared `application/octet-stream`, that no relationship reaches. Duplicate, colliding, unsafe, encrypted and over-limit entries go there too, by their compressed bytes, not decompressed. So "a part we could only partly read is rewritten only if the user edits it" holds like this: an unedited save returns it in place, and an edit keeps its bytes under `recovered/` while a valid form stands where it was.

**What stands in its place is what can be trusted.** A repair marks the elements it is inside. In a slide-like part, a shape or an optional child holding a mark is left out whole, and the rest is what arrived. So a slide with one broken shape keeps the others; a slide with none left, or none readable, is an empty slide marked damaged (`ImportedPart::damaged`, `ImportedSlides::damaged`). Other parts with a known role get an empty valid stand-in. A part with neither a role nor a readable reading outside a schema-bound namespace is removed with its links, and retained.

**A limit in a reader above is a repair too.** Import's limits, under `strict` a refusal with their own codes, set the part aside by default (`set_part_aside`).

### What a repair cannot see

- **Well-formed and merely wrong.** A value changed to another legal character, with a valid checksum, is not damage anyone can detect without validating against the schema, which the engine does not do. Such a part reads and is retained, as any third-party deck that breaks a rule is ([36](36-EXPORT-AND-SAVE.md)), and F4 reports it. The corpus's broken XML is what `walk` refuses, for this reason.
- **A checksum that fails on a part that still parses.** The bytes are kept and it is reported (`CS-REC-0104`), as the table above says. They may be wrong in a way that is schema-invalid. See the corpus's counts below for how often F4 sees it.
- **A relationship in a kept shape whose target is gone.** The `r:id` stays in the shape, and the link is reported. Rebuilt relationships cannot say what an old id meant.

## Evidence

The gate is a **damage corpus** generated from real decks: truncations, flipped bytes, removed parts, broken XML at many offsets. Every deck must open or be refused by one of the rules above, never panic or hang, and output valid files after an edit. Tracked as `REC-*` in [14](14-EXECUTION-TRACKER.md).

**Measured, 2026-11-07, sample mode** (`crates/casual-slide-export/tests/damage_corpus.rs`, the three decks in `fixtures/real` under 50 KB): 320 damaged decks. 318 opened. 2 were refused, both `CS-PKG-0005`, truncations that left no ZIP entry. No panic, no hang; the slowest deck took 3.2 seconds, unoptimized, on a loaded machine. An edit to each of the 318 was written and opens under `strict`.

**A flipped byte keeps its part (owner instruction 34).** F4 fails on outputs that hold a part with a flipped byte that still parses: the checksum fails (`CS-REC-0104`), and the bytes are kept and reported, since telling these apart needs validation against the schema, which the engine does not do, and a stand-in could wipe a whole master or theme over one byte. So such a file stays invalid until that part is repaired. The test accepts this only on outputs that carry `CS-REC-0104`; every other output must pass. **Measured, 2026-11-08, `DAMAGE_FULL=1 F4_REQUIRED=1`** (`tools/phase1-evidence.py damage`, recorded in `tools/phase1-evidence.json`): 16 decks in `fixtures/real`, 6203 damaged. 6200 opened, 2 refused `CS-PKG-0005`, no panic, no hang, slowest 1.35 seconds unoptimized at load 4 to 6. **It failed on one deck:** 32 bytes flipped in `z17310490-vierfelder-de`'s `slideLayout1.xml` local header at offset 9948 leave a package that opens from its directory, and the layout is found damaged only when read. That read repairs the package, which drops the layout and supplies one, but the slide import read on from its index of the package before the repair and failed with `CS-OPC-0001`. Fixed: when a part will not read, `import_slide_parts` indexes the repaired package again and reads from that (`a_layout_damaged_in_its_local_header_still_lets_the_slides_read`). F4 did not run on that run's outputs, since the test stopped at the broken rule. Neither has a campaign on the fuzz targets `xml_recover`, `package_salvage` and `deck_recover`, which have not been compiled either.

We cannot compare with what PowerPoint refuses until `PV-01`. Claims such as "opens what PowerPoint rejects" wait for that evidence.
