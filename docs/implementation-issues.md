# Implementation Issues

This project was useful as a sandbox, but the current implementation has a number of structural problems that would make it painful to extend into a more serious multiplayer game.

## Networking Model

- UDP is used as if it were reliable and ordered. Position updates, spawns, disconnects, and initial state sync all travel through the same mechanism with no sequence numbers, acknowledgements, retries, or deduplication.
- The client and server both spin in hot loops with no sleep or backpressure when there is no work. That wastes CPU and makes behavior dependent on machine speed.
- Message drops are expected and mostly handled by logging. Fixed-size `ArrayQueue` buffers silently become lossy transport under load.
- Disconnect handling is incomplete. The code has a `Disconnect` message path, but there is no real heartbeat, timeout, or socket liveness detection, so dead peers can linger forever.
- Client identity is tied directly to `SocketAddr`, which is fragile for any real deployment. NAT rebinding or reconnect behavior is not handled.

## Server Architecture

- The server stores global mutable state in `lazy_static` singletons instead of passing state explicitly. That makes the code harder to test, reason about, and evolve.
- `server_udp_networking` holds read locks while iterating client mailboxes and then awaits `send_to`. Holding shared state across async work is a bad pattern and will become a contention problem as the code grows.
- Connection bookkeeping, message transport, and game rules are tightly coupled. For example, `add_client` injects synthetic gameplay messages into the inbound queue.
- The server is effectively a relay with minimal validation. Clients can report arbitrary entity positions, and the server trusts them.
- Cleanup exists in `remove_client`, but it is not integrated into a full lifecycle. Resources can be allocated without a guaranteed removal path.

## Client Architecture

- `start_client.rs` declares both client and server modules in one binary crate. That works for a toy project, but it blurs boundaries and pulls unrelated code into the build.
- The client transmits `EntityPosition` every frame at a fixed rate with no interpolation, prediction, reconciliation, or bandwidth controls beyond the queue size.
- Input, simulation, networking, and rendering are mixed in one top-level loop. That makes it difficult to swap transport, test behavior, or separate local-only logic from replicated state.
- The local player spawn path is awkward. New clients request a spawn and then request all players immediately, which can create racey behavior when more state or more message types are introduced.

## Data And Protocol Design

- The protocol is ad hoc. There is no versioning, capability negotiation, or migration path for changing messages.
- Several message types are stateful events, but the system does not distinguish snapshot state from transient actions.
- `prune_latest_only_messages` exists as an idea, but it is disabled and would be risky as written because it globally rewrites the inbound queue without stronger ownership guarantees.

## Performance And Quality

- There are no tests around serialization, connection lifecycle, or gameplay state transitions.
- The codebase carries a lot of dead code and unused declarations, which makes the real execution path harder to identify.
- `Cargo.toml` still references bins that are no longer present, which is a sign that the repo is in archival rather than maintained shape.
- Logging is noisy and synchronous. The client can flood stdout with per-frame position messages, which is useful for debugging but not sustainable.

## If This Were Revived

The first cleanup pass should be:

1. Separate client and server crates or at least cleanly split modules.
2. Replace global singletons with explicit app state structs.
3. Introduce heartbeat and timeout handling.
4. Add a simple transport discipline: sequence numbers for state updates and explicit handling for unreliable vs reliable message classes.
5. Add basic integration tests for connect, spawn, move, disconnect, and reconnect flows.
