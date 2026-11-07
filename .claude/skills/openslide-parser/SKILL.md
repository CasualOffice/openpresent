---
name: openslide-parser
description: Security rules for code reading untrusted input in OpenSlide. Use when adding or changing anything that parses a file, a part, XML, an image reference, a font, or wire data.
---
Every byte is hostile, and we still open what we can (`docs/29-RECOVERY.md`): neutralize and report rather than refuse the deck.

Every byte is hostile. Read `docs/07-SECURITY-THREAT-MODEL.md`, and find the threats for this input.

- XML goes only through `casual-slide-ooxml`'s `walk` or `XmlTree`. `quick_xml` elsewhere fails `tools/check-repo.py`.
- Check declared sizes before decompressing or allocating. Never size an allocation from a file's number. Use `read_part_capped` for XML.
- Cap every count that comes from the file. Limits live in a limits struct, with a stable error code in `docs/20`.
- No panic paths: no `unwrap`, `expect`, slice indexing or plain arithmetic on file values. Use checked or saturating operations.
- No recursion following input without a depth bound. Graph walks need a visited set.
- External targets are classified and never followed. Nothing in the engine has network capability.
- Absent stays absent. An unknown value is retained and reported, and never coerced.

**Required with the change:** a hostile fixture per limit, in `fixtures/tools/generate.py`, with its expected code; a fuzz target under `fuzz/`, run for a short campaign; a `docs/07` row marked Tested only after the test was seen red.
