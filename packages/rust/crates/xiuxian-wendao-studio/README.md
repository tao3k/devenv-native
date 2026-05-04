# xiuxian-wendao-studio

`xiuxian-wendao-studio` owns the Studio-facing HTTP gateway adapter for Wendao.

This crate may depend on Wendao domain crates and on `xiuxian-wendao-server` for
Flight/gRPC transport contracts. The dependency direction remains one-way:
Studio adapters can call Wendao domain services and server transport contracts,
but `xiuxian-wendao` and `xiuxian-wendao-server` must not depend on this crate.

## Ownership

This crate owns:

- Studio HTTP route composition and handler state.
- Studio OpenAPI and route-contract exports.
- Studio Flight route providers backed by Wendao services.
- Frontend-facing API response shaping and gateway startup health checks.

`xiuxian-wendao-server` owns only the high-throughput Flight/gRPC transport
boundary. `xiuxian-wendao` continues to own graph, search, repository indexing,
parser, analyzer, and domain-runtime behavior.

## Feature Boundaries

The lightweight `contracts` feature owns route contracts and OpenAPI route
inventory. It must remain free of runtime gateway dependencies such as Axum,
Tonic, Arrow Flight, DuckDB, DataFusion, notify, `xiuxian-db-store`, and
`xiuxian-wendao-core`. Studio owns the frontend Specta type collection,
capability/search-manifest DTOs, and plugin artifact inspection DTOs. Plugin
artifact DTO conversion is compiled only with `local-runtime` because it
depends on runtime plugin payload records. The remaining DTO definitions still
depend on the full local runtime through the domain transition re-export and
remain the next convergence target.

Runtime concerns are layered behind explicit features:

- `http-router`: Axum/Tower router and handler composition.
- `flight-transport`: Arrow Flight and gRPC provider adapters.
- `local-runtime`: repository indexing, search-plane, DuckDB/DataFusion,
  watcher, parser, and local project integration.
- `studio`: full Studio composition.
- `cli-bin-support`: binary-only support for commands that require the full
  Studio runtime.

`local-runtime` uses `xiuxian-zhenfa` only through its native registry surface.
The HTTP gateway/client features remain opt-in through `zhenfa-router` or
`cli-bin-support`, so Studio's local runtime does not inherit Zhenfa HTTP
composition by accident.
