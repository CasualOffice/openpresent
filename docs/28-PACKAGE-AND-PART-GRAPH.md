# 28 — Package and part graph (Tier 1A)

`casual-slide-ooxml` admits a package and reports its structure. It maps nothing into the model.

- **One XML path.** `xml::walk` is the only way in; `XmlTree::parse` runs it first, then reads a second time. It refuses any DOCTYPE and undeclared entities, and bounds size, elements, depth and attributes. `tools/check-repo.py` fails if `quick_xml` appears elsewhere.
- **Opening reads structure only:** content types, `.rels`, and the main part, which is walked in full. Media is never read, so no decoder is reachable by opening a file.
- **The graph** is a breadth-first walk with a visited set. A cycle costs one visit.
- **Nothing external is followed.** Targets are classified after removing the whitespace browsers ignore, and reported.
- **Findings do not stop a deck opening,** and everything they name is retained. Codes are in [20](20-ERROR-CODE-REGISTRY.md).
- **Saving:** unedited returns the input bytes. Edited copies other entries raw, keeping their compressed bytes.
- **Main part:** matched on exact relationship namespaces, so a look-alike type is not believed. Strict and Transitional are both **detected**; handling Strict is `STRICT-01`.

## Editing inside a part: `XmlTree` (ADR-013)

Rebuilding XML from a typed model loses whatever the model does not hold: unknown attributes, `extLst`, ordering, prefixes, whitespace. Holding those as "retained fragments" beside the model is a long tail of special cases.

So each part read for editing becomes a tree that keeps its source text, and writes back byte for byte. **`-model` holds plain typed values and never sees XML. `-import` fills it from the tree. `-export` keeps the tree: ops mark objects dirty, and on save it writes each dirty object's values to the nodes they came from, so only those change.** (`ARC-01`) Every XML part in the corpus, third-party decks included, is tested to round-trip exactly. The cost is memory per open part, which lazy per-slide parsing is to bound; that is not built yet. `SPEC-01`'s audit still measures what the typed view covers.
