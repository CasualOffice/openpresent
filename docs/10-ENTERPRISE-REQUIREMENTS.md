# 10 — Enterprise requirements

**This is the bar, set by the owner: an enterprise solution, not an MVP.** Requirements here are gates
on the phases that deliver them, and they shape design from the first crate. Compliance certifications
such as SOC 2 are organizational, so the product is built to support them and does not claim them.

| ID | Area | Requirement | Designed in | Delivered |
| --- | --- | --- | --- | --- |
| ENT-01 | Identity | SSO through the host: OIDC and SAML, signed tokens; no password store of our own | 9 | 10 |
| ENT-02 | Access | View, comment, edit enforced on the server per operation; link and group sharing belong to the host | 4 | 9 |
| ENT-03 | Audit | Append-only events: who opened, edited, shared, exported, downloaded. Export to SIEM | 9 | 10 |
| ENT-04 | Classification | Read, display and preserve sensitivity labels. Host policy enforces no-download and no-print | 1M | 10 |
| ENT-05 | Encryption | Open password-protected decks, client-side. At rest and in transit encryption with host-managed keys | 1A | 10 |
| ENT-06 | IRM | Rights-protected files need a rights service. Detect, refuse clearly, retain bytes; integrate behind a host seam | 1A | 10 |
| ENT-07 | Data residency | Self-hostable, including air-gapped: no external calls, CDN, fonts or telemetry by default | 0 | 3 |
| ENT-08 | DLP | Hooks on export, print, copy and download, so a host can apply policy | 3 | 10 |
| ENT-09 | Retention | Version history and legal hold through host seams; deletion honoured | 9 | 10 |
| ENT-10 | Availability | No sticky sessions; leader failover; resume without loss; documented RPO and RTO, as OpenCalc's stack | 9 | 9 |
| ENT-11 | Scale | Published capacity numbers from load tests on a named environment | 3 | 9 |
| ENT-12 | Operations | Helm chart, health checks, metrics, structured logs, zero-downtime upgrades, backward-compatible protocol | 9 | 10 |
| ENT-13 | Accessibility | WCAG 2.2 AA, Section 508, EN 301 549. Screen reader, keyboard-only, high contrast. Accessibility checker. Tagged PDF | 3 | every UI phase |
| ENT-14 | Internationalization | Localized UI, RTL UI and content, CJK, IME; locale-aware numbers and dates in fields | 2D | 5, 10 |
| ENT-15 | Compatibility | Published matrix of browsers and PowerPoint versions; files open without a repair prompt | 1 | every phase |
| ENT-16 | Integration | WOPI for SharePoint, Nextcloud and others; embeddable SDK with stable versioned API; REST; MCP | 3 | 10 |
| ENT-17 | Branding | Host-themed, white-label, command hiding, custom fonts and templates through an organization library | 3 | 10 |
| ENT-18 | Supply chain | Reproducible builds, signed releases, SBOM, licence policy gate, disclosure process, supported-version policy | 0 | 10 |
| ENT-19 | Assurance | Third-party penetration test before the first stable release; continuous fuzzing; threat model kept current | 0 | 10 |
| ENT-20 | Support | Semantic versioning, deprecation policy, long-term-support line, security-fix SLA, migration notes | 10 | 10 |
| ENT-21 | Trust in output | Every save names anything it could not keep; exports are deterministic and auditable | 1 | 1 |
| ENT-22 | Admin | Tenant settings: default fonts, templates, external content policy, macro notice, feature toggles | 3 | 10 |
| ENT-23 | Quality | No P0 or P1 open at release; coverage of the spec checklist published per release | 0 | every phase |
| ENT-24 | Legal | Rights review for bundled fonts, schemas, fixtures and templates; patent position reviewed | 0 | 10 |

## What this changes in the engine, now

- **Licences and bundled assets are inventoried from Phase 0;** the legal review (`DEC-07`) must finish before first public release.
- **No network capability in the engine at all**, which is what makes air-gapped deployment and
  server-side rendering safe by construction.
- **Every host-specific behaviour is a seam from the start,** so ENT-04, 06, 08 and 09 add behaviour
  rather than a rewrite.
- **Diagnostics must never contain deck content,** so logs can be shipped to a SIEM. Not yet true (`SEC-015`).
