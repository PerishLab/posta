# Cold-start verification boundary

Passing means the first postal loop is closed by the library and Keel
projections.

## Layers

| Layer | Evidence |
|---|---|
| L1 | model and ceremony unit behavior |
| L2 | real SQLite reopen plus in-process Keel Router |
| L3 | fmt, clippy, Ectropy, and Plumb |

## Must pass

- A request stages one outbound letter under one mailbox.
- Exact request replay returns the same letter.
- Changed content under the same request refuses.
- A reply may arrive before its remote thread is known.
- Exact early reply replay remains one durable pending fact.
- Receipt acceptance creates the thread and settles pending replies atomically.
- Reopening the same Keel estate preserves and completes the loop.
- Exact settled reply replay returns the same letter.
- Changed content under the same cause refuses.
- Keel projects public postal resources through its in-process Router.
- Veiled pending facts have no HTTP route.
- The repository guard is green.

## Out

- Credential admission and signed capabilities.
- Santi or Stim dependency migration.
- A Posta binary, listener, configuration, or hosted service.
- Independent prose or cross-language schema.
- Group membership, attachments, edits, reactions, and federation.
