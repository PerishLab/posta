# Agents

`posta` is a library-first postal protocol built on Keel. Product callers
declare runtime policy and credential custody. Posta owns postal vocabulary and
transactional ceremony; Keel owns generic resource mechanism and projections.

## Boundary

- `graph()` is the canonical persistent model.
- The `posta` crate version is the released schema version.
- `Store` is the invariant-preserving protocol face.
- HTTP is Keel's projection of the graph, not a Posta authoring surface.
- Runtime configuration, listeners, credentials, and deployment stay with the
  caller.
- A missing generic resource mechanism pauses Posta and lands in Keel first.
- No binary or independent prose specification is required during cold start.

## Laws

- Persisted protocol facts are immutable where Keel can enforce immutability.
- Replay returns the existing fact only when the complete semantic payload
  agrees; changed payload under the same key refuses as conflict.
- Early replies remain durable and veiled until their thread becomes known.
- One Keel transaction closes every multi-fact protocol transition.
- Unknown, malformed, conflicting, or drifted state refuses.
- Single word, block depth <= 4, path depth <= 4, comments denied by default.

## Operating

- Never commit on `main`.
- Run `runseal :guard` before land.
- Land only through `runseal :land`.
- Operator wrappers under `.runseal/` are TypeScript.

## Verification

Cold-start closure is `docs/run/verify.md`. The L2 scenario drives Keel's axum
Router in process and reopens a real SQLite estate.
