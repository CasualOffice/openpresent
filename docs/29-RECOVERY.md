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

## Evidence

The gate is a **damage corpus** generated from real decks: truncations, flipped bytes, removed parts, broken XML at many offsets. Every deck must open or be refused by one of the rules above, never panic or hang, and output valid files after an edit. Tracked as `REC-*` in [14](14-EXECUTION-TRACKER.md).

We cannot compare with what PowerPoint refuses until `PV-01`. Claims such as "opens what PowerPoint rejects" wait for that evidence.
