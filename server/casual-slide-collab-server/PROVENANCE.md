# Provenance

Started from `server/casual-calc-collab-server` in OpenCalc (`../opencalc`) at commit
`b23c67c`, on 2026-11-08 (owner instruction 42, `VS-09`). Same owner, Apache-2.0.

Taken, and changed only where slides need it:

- The shape of the service: one WebSocket per participant at `/collab`, the frame limit set
  on the upgrade before a byte is read, a first message that must be a join within a bound
  of time, presence relayed and never logged, `Departed` when a participant goes, a ping
  answered with a pong, `/healthz` (`net.rs`).
- Ordering: one session per document; a chunk is rebased over everything committed since
  its base, acknowledged to its sender and relayed as it landed to everyone else
  (`document.rs`, and `ServerSession` from its transaction crate, now generic in
  `casual-slide-transaction::collab`).
- The participant colours and their rule, deterministic in the id (`presence.rs`).
- Separate cargo workspace, as OpenCalc keeps its async stack out of the engine (its
  ADR-023).

Changed: the server holds no model. OpenCalc's server imports its model, import, export,
eval and transaction crates and applies every operation to a workbook; this one sees only
the seam (`Engine`: wire operations and transform) and keeps the seed deck's bytes and the
ordered log. A room is seeded by its first participant and reached by an unguessable key in
its link, since there is no integrator to fetch a document from yet.

Not taken (the audit in `docs/24` §The server): token verification (`token.rs`,
`verify.rs`), the integrator's document fetch and save-back (`http.rs`, `lifecycle.rs`), the
Redis cluster, leases and relay between nodes (`cluster/`, `relay.rs`), resume, memory
admission, metrics and TLS. Each is a row in `docs/14` (`COL-03` to `COL-05`).
