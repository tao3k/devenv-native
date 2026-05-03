# xiuxian-wendao-studio

`xiuxian-wendao-studio` owns the Studio-facing HTTP gateway adapter for Wendao.

This crate may depend on Wendao domain crates and on `xiuxian-wendao-web` for
Flight/gRPC transport contracts. The dependency direction remains one-way:
Studio adapters can call Wendao domain services and web transport contracts,
but `xiuxian-wendao` and `xiuxian-wendao-web` must not depend on this crate.

## Ownership

This crate owns:

- Studio HTTP route composition and handler state.
- Studio OpenAPI and route-contract exports.
- Studio Flight route providers backed by Wendao services.
- Frontend-facing API response shaping and gateway startup health checks.

`xiuxian-wendao-web` owns only the high-throughput Flight/gRPC transport
boundary. `xiuxian-wendao` continues to own graph, search, repository indexing,
parser, analyzer, and domain-runtime behavior.
