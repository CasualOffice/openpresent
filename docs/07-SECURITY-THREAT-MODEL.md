# 07 — Security threat model

**A deck is hostile until proven otherwise.** It comes from email, from strangers, and from people
trying to reach the next person who opens it. Every threat here has a control and a named test. A
limit without a test seen to fail is not a control.

Status: **Tested** means a test exists and was seen red without the fix. **Planned** names where it
lands; some planned rows have partial code, and only Tested rows may be relied on. **Partial** means a control exists but no test proves all of it.

## Package

| ID | Threat | Control | Status |
| --- | --- | --- | --- |
| SEC-P01 | Zip bomb by ratio or total size | Admission caps; `rejects_zip_bomb_by_expansion_ratio`, `rejects_expansion_over_total_cap` | Tested |
| SEC-P02 | Forged sizes: part produces more than it declared | Bytes held to the declaration. **The `zip` crate enforces this first**: since version 9 it stops a read past the declared size, and `read_part` maps that error to `CS-PKG-0003` (`a_part_that_produces_more_than_it_declared_is_refused`). Our own bound (`take(cap + 1)`) is a second line, which no test can turn red while `zip` holds; if a `zip` release drops its check or rewords its error, that test turns red. A whole-package budget is charged on read, once per part (`PKG-01`, `rereading_a_part_does_not_spend_the_budget_again`) | Tested |
| SEC-P03 | Path traversal, absolute or drive paths | `is_safe_part_path` | Tested |
| SEC-P04 | Declared size used to allocate; a padding entry satisfies the ratio; in WASM a failed allocation aborts the instance | Up-front capacity capped at 1 MiB, observed on the real read: `read_part_allocates_up_front_no_more_than_the_bound` | Tested |
| SEC-P05 | Duplicate entries: a scanner approves one, the editor opens another. The `zip` crate hides them | Raw walk of the central directory compared with the crate's count | Tested |
| SEC-P06 | Part names differing only in case | Refused, per OPC equivalence, which folds ASCII letters only: `É` and `é` are two parts. The package and the part graph share one rule, `part_name_key` (`PKG-02`): `parts_differing_only_in_case_are_refused`, `letters_outside_ascii_are_not_folded`, `percent_encoded_spellings_are_one_part`, fixture `non-ascii-case-distinct-parts` | Tested |
| SEC-P07 | Malformed names: `.`, empty segment, trailing dot, control characters, `%2e`, `%2f`, `%5c` | Refused | Tested |
| SEC-P12 | Zip bomb diluted by ordinary parts, so the overall ratio passes | Ratio also checked per entry. Found by the corpus: a 64 MiB run of zeros was admitted | Tested |
| SEC-P13 | Part decompressed before its size is judged | XML reads refuse on the declaration, with nothing decompressed: on open, in `read_resolved`, and through `PresentationPackage::read_xml_part`, which import and export use for every part they parse (`PKG-04`). `an_oversized_xml_part_is_refused_before_it_is_read`, `an_oversized_optional_part_is_not_decompressed`; opening charges no media, `opening_a_deck_does_not_read_its_media` asserts the read ledger | Tested |
| SEC-P14 | One part under two spellings (`a b.png`, `a%20b.png`): a scanner approves one, the editor opens the other. A target that decodes to traversal: `%2e%2e`, `%252e`, overlong `%c0%ae`, `..\`, look-alike dots and slashes | ECMA-376 Part 2 equivalence in `part_name_key`. Two entries for one part refused as `CS-PKG-0007`; a disguised target or entry reported as `CS-FND-0011`, and not followed; `two_entries_naming_one_part_are_refused`, `what_decodes_to_traversal_is_refused`, `a_target_that_decodes_to_traversal_is_reported_and_not_followed` | Tested |
| SEC-P08 | Encrypted or unusual compression methods in entries | Refuse cleanly on read; hostile fixture | Planned 0 |
| SEC-P09 | Recursive bomb: embedded `.xlsx` or `.pptx` inside a deck | Embedded parts are retained, never auto-opened; opening one is a separate bounded admission with an inherited budget | Planned 1J, 1L |
| SEC-P10 | Slow decompression within the size budget | Time budget, cancellable; host-supplied predicate | Planned 1A |
| SEC-P11 | Local header disagreeing with central directory | Compare name and sizes on read | Planned 0 |

## XML

| ID | Threat | Control | Status |
| --- | --- | --- | --- |
| SEC-X01 | Billion laughs, entity expansion | **Any DOCTYPE is refused**; OOXML never needs one. Undeclared entity references also refused | Tested |
| SEC-X02 | XXE, SSRF through external entities | Same, and the parser has no resolver | Tested |
| SEC-X03 | Deep nesting exhausts the stack | Depth cap; parsers iterative where recursion follows input | Tested |
| SEC-X04 | Element, attribute or text volume | Per-part caps on size, elements and attributes. Each seen red with its cap removed: fixtures `xml-too-many-elements`, `xml-too-many-attributes`, `xml-part-over-size`; the walker's own size, element and attribute bounds in `the_walker_refuses_*` | Tested |
| SEC-X05 | `mc:AlternateContent` used to hide content from a scanner | Choice resolved by a fixed rule over namespace URIs, never prefixes; retained branch is never interpreted, and is named in the report; a malformed construct is never selected. A must-understand namespace we do not know refuses as `CS-MCE-0001` where it is read. `the_first_choice_whose_requirements_are_understood_wins`, `prefixes_are_not_believed`, `a_branch_not_taken_is_retained_and_never_interpreted`, `broken_constructs_are_reported_and_not_guessed_at`. Applied to the main part on open, and to other parts by `read_resolved` | Tested |
| SEC-X06 | Relationship cycles or a part reached many ways | Visited set tested. Caps on relationships per part and in total, and on content types, each seen red with its cap removed: fixtures `rels-per-part`, `rels-total`, `content-types-flood` | Tested |
| SEC-X07 | Theme, master or layout chains that recurse | Fixed walk order; cycle refused | Planned 1D |
| SEC-X08 | Readers that disagree about the same bytes: a malformed or repeated attribute, `a="1"b="2"`, NUL or other characters XML forbids, an undeclared prefix, text outside the root. One reader skips what another reads | `walk` admits only namespace-well-formed XML 1.0, and the tree runs it first, so every reader agrees or none reads (`XML-02`, `XML-03`). Attributes are read by the name written, never by local name in any namespace (`XML-01`). `malformed_and_repeated_attributes_are_refused`, `what_is_not_xml_is_refused`, fixtures `xml-*`; fuzz target `xml_tree` holds walker and tree to one answer | Tested |
| SEC-X09 | Document text reaching logs through a diagnostic | A refusal's reason is fixed text; no error carries a value read from a part; quick-xml's messages, which quote the document, are never used (`SEC-015`). `no_message_or_finding_carries_document_text` produces every `CS-PKG`, `CS-XML`, `CS-OPC`, `CS-MCE` and `CS-FND` code with a marker planted. Import's own codes are not yet covered | Tested |

## Content

| ID | Threat | Control | Status |
| --- | --- | --- | --- |
| SEC-C01 | Image bomb: small file, enormous pixel count | Read dimensions first; pixel cap before decode; decode in a worker | Planned 1H, 3 |
| SEC-C02 | Decoder bugs in PNG, JPEG, GIF, TIFF, WebP | Browser decodes in the editor. Native uses pure-Rust decoders with features named one by one | Planned 2E |
| SEC-C03 | **SVG with script, external references or `foreignObject`.** A classic XSS vector | Never placed in the DOM as SVG. Sanitized and rendered from our own path model, or as an `<img>` in a sandboxed context | Planned 1H, 2E |
| SEC-C04 | **EMF and WMF**, with a long CVE history, common in real decks | Parsed by our own bounded reader over a drawing-record allowlist; else the fallback image | Planned 2E |
| SEC-C05 | Font files: malformed tables; embedded `.fntdata` | Embedded fonts off by default for rendering; embedding rights honoured; `skrifa` only, fuzzed | Planned 2D |
| SEC-C06 | Media: video and audio parsers | Browser plays in a sandbox; the engine never parses media | Planned 3 |
| SEC-C07 | VBA, ActiveX, OLE | **Never executed.** Retained as bytes; `.pptm` opens with a visible notice; fallback image only | Planned 1L |
| SEC-C08 | Geometry guide formulas: cycles, huge values, division by zero | Evaluated in document order with no recursion; saturating arithmetic; cap on guides and path points. Import compiles every custom geometry under those caps, over `DrawingLimits` (`CS-DRW-0001`), with hostile fixtures and the `drawing_properties` fuzz target (`GEO-04`) | Tested |
| SEC-C09 | Pathological text: combining floods, huge runs, deep lists | Per-body caps; layout time budget | Planned 2D |
| SEC-C10 | Animation or transition trees that never end or explode | Depth and node caps; timeline compile time budget; infinite repeat bounded per show | Planned 1I, 8 |
| SEC-C11 | Effects costing unbounded CPU, such as huge blur radii | Radius and rasterized-area caps per object | Planned 2C |
| SEC-C12 | SmartArt layout definitions and chart data at huge scale | Render from stored drawing; never run layout definitions on load; point caps | Planned 1J, 1K |
| SEC-C13 | Text bodies: a paragraph or run count sized to exhaust memory; a tab list far beyond the schema; a bad value in any of some hundred attributes; document text reaching a report | `TextLimits`, each bound refusing with its own code; bad values reported and left out, with the tree keeping them; findings carry names only. `hostile/text-*.bin`, `hostile_text_refuses_with_its_code`, `no_message_or_finding_carries_text_from_a_body`, fuzz target `text_bodies`. A body's warp and 3D, a `p:style` and a theme's object defaults go through the same readers: hostile `text-limit-warp-guides`, `text-limit-3d-depth`, `text-bad-3d`, `shape-style-bad-values`, `theme-object-defaults-bad-values`; both `text_bodies` and `drawing_properties` fuzz them | Tested |

## Links and network

| ID | Threat | Control | Status |
| --- | --- | --- | --- |
| SEC-N01 | **Tracking and SSRF:** `TargetMode="External"` images, media, OLE links, remote templates | **Never fetched automatically**, on the server especially; every one reported. Host policy and placeholder come in Phase 3 | Tested for 1A |
| SEC-N02 | NTLM credential theft through UNC paths such as `\\host\share` | UNC and `file:` targets classified, reported, never followed; inert display comes in Phase 3 | Tested for 1A |
| SEC-N03 | Link schemes: `javascript:`, `data:`, `vbscript:`, `file:` | Allowlist: `http`, `https`, `mailto`, `tel`, and in-deck jumps. Confirmation before leaving | Planned 3 |
| SEC-N04 | `ppaction://` program launch and macro actions | Parsed and retained; program and macro actions never run | Planned 1D, 8 |
| SEC-N05 | Link text that lies about its target | Tooltip shows the real target | Planned 3 |
| SEC-N06 | Server-side rendering reaching internal networks | The render path has no network capability at all | Planned 2 |

## Editor

| ID | Threat | Control | Status |
| --- | --- | --- | --- |
| SEC-E01 | XSS from document strings: shape names, alt text, comments, tags, notes | `textContent` only; no HTML from documents; lint gate against `innerHTML` with document data | Planned 3 |
| SEC-E02 | Clipboard: pasted HTML and SVG | Sanitized to the supported model; no script survives | Planned 4 |
| SEC-E03 | Injection through third-party page content | Strict CSP with no inline script; Trusted Types; WASM served from the host's origin | Planned 3 |
| SEC-E04 | Embed: clickjacking, host and frame confusion | `frame-ancestors` by policy; origin-checked `postMessage` | Planned 10 |
| SEC-E05 | Prompt injection through deck text via MCP | Deck content is data, marked as untrusted in tool results | Planned 10 |

## Service

| ID | Threat | Control | Status |
| --- | --- | --- | --- |
| SEC-S01 | Broken access: view-only user submits edits | Enforced on the server per op, not by hiding buttons | Planned 9 |
| SEC-S02 | Token theft or replay | Short-lived signed tokens, audience-bound, revocable; no token in URLs where avoidable | Planned 9 |
| SEC-S03 | Malicious ops | Same validation as local ops; frame and rate caps; transform bounded | Planned 9 |
| SEC-S04 | WOPI SSRF, forged host | `ALLOWED_HOSTS` is required; proof-key validation | Planned 10 |
| SEC-S05 | Secrets or deck content in logs | Diagnostics carry codes and counts, not content; a gate checks | Planned 9 |
| SEC-S06 | Tenant bleed in cache or memory | Per-document isolation; no shared glyph or image cache across tenants on the server | Planned 9 |
| SEC-S07 | Supply chain | `cargo deny`, `cargo audit`, lockfiles, SBOM, pinned Actions, no CDN | Planned 0 |

## Encrypted and signed files

Password-protected decks are a Compound File wrapping an encrypted package ([MS-OFFCRYPTO]). The CFB
reader is a new hostile-input parser with its own fuzz target and limits. The password is never logged
or sent to a server, and decryption runs in the client. Editing a signed deck invalidates its signature,
so the user is told and the signature parts are dropped explicitly, in the report, not silently.

## Standing rules

- **Resource exhaustion ends in a clean refusal,** never a crash, hang or OOM.
- **Copied code is held to this document.** `SEC-P04` to `SEC-P07` arrived with OpenCalc's crate.
- **A fuzz target accompanies every parser,** and a crash found by one becomes a fixture.
- **A bound is never quietly lowered in prose.** A contract the code does not keep is a tracker row.
