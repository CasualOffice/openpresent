# 28 — Package and part graph (Tier 1A)

`casual-slide-ooxml` admits a package and reports its structure. It maps nothing into the model.

- **One XML path.** `xml::walk` is the only reader. It refuses any DOCTYPE and undeclared entities, and bounds size, elements, depth and attributes. `tools/check-repo.py` fails if `quick_xml` appears elsewhere.
- **Opening reads structure only:** content types, `.rels`, and the main part's root. Media is never read, so no decoder is reachable by opening a file.
- **The graph** is a breadth-first walk with a visited set. A cycle costs one visit.
- **Nothing external is followed.** Targets are classified after removing the whitespace browsers ignore, and reported.
- **Findings do not stop a deck opening,** and everything they name is retained. Codes are in [20](20-ERROR-CODE-REGISTRY.md).
- **Saving:** unedited returns the input bytes. Edited copies other entries raw, keeping their compressed bytes.
- **Main part:** matched on exact relationship namespaces, so a look-alike type is not believed. Strict and Transitional are both read.
