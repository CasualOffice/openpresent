# MCE: Markup compatibility (ECMA-376 Part 3)

Generated. Tier 1A; render -; edit -; level Retain. 9 rows.

Written by hand, since ECMA-376 Part 3 ships no XSD: `tools/spec-checklist/markup-compatibility.tsv`. Tied to `BEH-18` in [51](../51-SPEC-CHECKLIST-AND-FIDELITY.md). Built in `crates/casual-slide-ooxml/src/mce.rs`.


## `mc:<AlternateContent>`
Restriction: `Part 3 9.1. Takes the first Choice whose Requires are understood, else Fallback. Both branches stay in the part`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `Choice` | `` | 1..unbounded |  | The first that applies is taken. At least one, before any Fallback |
| element | `Fallback` | `` | 0..1 |  | Taken when no Choice applies. At most one, and last |

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@Requires` | `prefix list` | required |  | Every prefix must be declared, and must name a namespace this engine understands |

## `mc:@Ignorable` : `prefix list`
Restriction: `Namespaces a consumer not understanding them may skip. Scoped to the element and its descendants`

## `mc:@ProcessContent` : `qname list`
Restriction: `Ignorable elements whose content is kept and processed. Must name an ignorable namespace`

## `mc:@MustUnderstand` : `prefix list`
Restriction: `A namespace not understood here stops processing the part, as CS-MCE-0001`

## `mc:@PreserveElements` : `qname list`
Restriction: `Named by Part 3 for a consumer that resaves. Accepted; the part's bytes are kept anyway (ADR-013)`

## `mc:@PreserveAttributes` : `qname list`
Restriction: `As PreserveElements`
