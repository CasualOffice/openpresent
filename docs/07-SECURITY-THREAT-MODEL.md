# 07 — Security threat model

**A deck is hostile until proven otherwise.** It comes from email, from strangers, and from people
trying to reach the next person who opens it. Every threat here has a control and a named test. A
limit without a test seen to fail is not a control.

Status: **Tested** means a test exists and was seen red without the fix. **Planned** names where it
lands; nothing marked Planned exists yet.

## Package

| ID | Threat | Control | Status |
| --- | --- | --- | --- |
| SEC-P01 | Zip bomb by ratio or total size | Admission caps; `rejects_zip_bomb_by_expansion_ratio`, `rejects_expansion_over_total_cap` | Tested |
| SEC-P02 | Forged sizes: part produces more than it declared | Bytes held to the declaration, whole-package budget charged on read | Tested |
| SEC-P03 | Path traversal, absolute or drive paths | `is_safe_part_path` | Tested |
| SEC-P04 | Declared size used to allocate; a padding entry satisfies the ratio; in WASM a failed allocation aborts the instance | Up-front capacity capped at 1 MiB; grows as bytes arrive | Tested |
| SEC-P05 | Duplicate entries: a scanner approves one, the editor opens another. The `zip` crate hides them | Raw walk of the central directory compared with the crate's count | Tested |
| SEC-P06 | Part names differing only in case | Refused, per OPC equivalence | Tested |
| SEC-P07 | Malformed names: `.`, empty segment, trailing dot, control characters, `%2e`, `%2f`, `%5c` | Refused | Tested |
| SEC-P12 | Zip bomb diluted by ordinary parts, so the overall ratio passes | Ratio also checked per entry. Found by the corpus: a 64 MiB run of zeros was admitted | Tested |
| SEC-P13 | Part decompressed before its size is judged | XML reads refuse on the declaration, with nothing decompressed | Tested |
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
| SEC-X04 | Element, attribute or text volume | Per-part caps on size, elements and attributes | Tested |
| SEC-X05 | `mc:AlternateContent` used to hide content from a scanner | Choice resolved by a fixed rule; retained branch is never interpreted | Planned 1A |
| SEC-X06 | Relationship cycles or a part reached many ways | Breadth-first walk with a visited set, relationship caps | Tested |
| SEC-X07 | Theme, master or layout chains that recurse | Fixed walk order; cycle refused | Planned 1D |

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
| SEC-C08 | Geometry guide formulas: cycles, huge values, division by zero | Evaluated in document order with no recursion; saturating arithmetic; cap on guides and path points | Planned 1E |
| SEC-C09 | Pathological text: combining floods, huge runs, deep lists | Per-body caps; layout time budget | Planned 2D |
| SEC-C10 | Animation or transition trees that never end or explode | Depth and node caps; timeline compile time budget; infinite repeat bounded per show | Planned 1I, 8 |
| SEC-C11 | Effects costing unbounded CPU, such as huge blur radii | Radius and rasterized-area caps per object | Planned 2C |
| SEC-C12 | SmartArt layout definitions and chart data at huge scale | Render from stored drawing; never run layout definitions on load; point caps | Planned 1J, 1K |

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
