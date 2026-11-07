# Security policy

Adapted from OpenCalc's `SECURITY.md` at `b23c67c`.

## Reporting

Do not open a public issue for a suspected vulnerability, a malicious fixture, or a
confidential deck.

Use [GitHub private vulnerability reporting](https://github.com/CasualOffice/openpresent/security/advisories/new)
for this repository. Include:

- the affected revision or release;
- the affected subsystem and host (web WebAssembly, desktop, headless);
- the impact and a realistic attack path;
- a minimal reproduction **without confidential content**: a deck cut down to the
  parts that trigger it, or a script that builds one;
- whether exploitation is known;
- a suggested mitigation, if you have one.

We will acknowledge a complete report, assess its severity, coordinate a fix and an
advisory, and credit the reporter if asked. Please do not publish details before
coordinated disclosure.

## Supported versions

There is no release. The product (OpenSlide, being renamed OpenPresent) is pre-alpha:
nothing is published, nothing renders yet, and there is no editor. Security fixes
target the default branch. Supported release lines and their end-of-support dates will
be listed here before the first public preview.

The package reader already admits untrusted decks under the bounds below, and those
bounds are tested and fuzzed. **A parser bug is in scope for this policy today.**

## Security boundaries

- Decks, templates, shows, packages, XML parts, relationships, images, media, fonts and
  operation logs are **untrusted input**. Every file is hostile until proven otherwise.
- **No automatic network fetch.** Linked images, linked media, remote templates and
  external relationships are reported, never followed.
- **Macros never run.** A VBA project is kept as opaque bytes and reported.
- **Embedded objects never run.** OLE payloads are kept as opaque bytes.
- Parser and resource limits are **required behaviour**, not best effort:
  [docs/21](docs/21-PARSER-LIMITS.md) lists every one, with its error code.
- Diagnostics name parts and counts, **never document content**
  ([docs/20](docs/20-ERROR-CODE-REGISTRY.md)). Where that contract is known to be broken,
  a tracker row says so (`SEC-015`).
- No document string reaches the DOM as HTML.

The threat model, each threat's control and its test are in
[docs/07](docs/07-SECURITY-THREAT-MODEL.md).
