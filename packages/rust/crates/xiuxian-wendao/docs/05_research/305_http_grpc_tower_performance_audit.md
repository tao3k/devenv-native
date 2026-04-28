# HTTP, gRPC, and Tower Performance Audit

:PROPERTIES:
:ID: research-wendao-http-grpc-tower-performance-audit
:TYPE: RESEARCH
:STATUS: ACTIVE
:DATE: 2026-04-28
:END:

## Purpose

This note records the Wendao HTTP, gRPC, and Tower performance audit prompted by
Michael Snoyman's article
[Combining Axum, Hyper, Tonic, and Tower for hybrid web/gRPC apps: Part 4](https://academy.fpblock.com/blog/axum-hyper-tonic-tower-part4/).

The goal is not to copy the article's custom Hyper hybrid service. The goal is
to use its Tower service model as an audit lens for Wendao's current Axum,
Tonic, and Arrow Flight transport surfaces.

This audit does not change the public API boundary: Wendao's external caller
surface should remain HTTPS JSON, with server-sent events for user-facing
streaming responses. Arrow Flight, FlightSQL, gRPC-Web, and HTTP/1 compatibility
are deployment-scoped transport surfaces, not the default public API shape.

## External Design Signal

The article's relevant design observations are:

1. HTTP and gRPC can share one listener because gRPC is HTTP/2-based.
2. A hybrid service has two asynchronous levels:
   connection-level service construction and request-level service execution.
3. Tower readiness matters. A correct hybrid service must account for
   `poll_ready`, request execution, response body unification, and error
   unification.
4. gRPC requests are easy to distinguish at the HTTP layer because they carry
   `content-type: application/grpc`.
5. Type-erased services can reduce implementation complexity, while enum-based
   future and body unification may avoid dynamic dispatch.

Wendao currently uses a more pragmatic modern Axum route-mounting strategy
instead of a custom Hyper hybrid make-service. That is acceptable. The
performance risk is therefore not "missing the custom hybrid service"; the risk
is whether each mounted transport branch has explicit backpressure, timeout,
and concurrency semantics.

## Current Wendao Shape

The primary gateway surface is the CLI gateway router in
`src/bin/wendao/execute/gateway/command.rs`.

- `build_gateway_router` builds the shared Axum router.
- The Studio HTTP router is wrapped with `ServiceBuilder`, `load_shed`,
  `timeout`, and `concurrency_limit`.
- The Arrow Flight business plane is mounted on the same Axum listener through
  `any_service` when the `zhenfa-router` feature is enabled.
- `mount_gateway_flight_service` can wrap the `FlightServiceServer` with
  `GrpcWebLayer` when compatibility is explicitly enabled, and mounts it at
  `/arrow.flight.protocol.FlightService/{*grpc_method}`.

There are also standalone Flight server binaries:

- `src/bin/wendao_search_flight_server.rs` starts a Studio-backed Flight server
  that can opt into `accept_http1(true)` and `GrpcWebLayer`.
- `packages/rust/crates/xiuxian-wendao-runtime/src/bin/wendao_flight_server.rs`
  starts a runtime-owned Flight server without the gRPC-Web compatibility
  layer.

The runtime Arrow Flight client lives in
`packages/rust/crates/xiuxian-wendao-runtime/src/transport/flight.rs`.

## Findings

### P1: Flight client concurrency is likely serialized by a client mutex

`ArrowFlightTransportClient` has an explicit in-flight request gate, but the
current implementation stores one `FlightClient` behind a `Mutex` and holds the
lock across `do_exchange().await`.

Impact:

- `max_in_flight_requests` can admit multiple tasks, but the shared client lock
  can serialize the actual transport call.
- This weakens the intended throughput contract.
- The issue is directly on the hot Arrow Flight client path, so it should be
  treated as the first performance-hardening slice.

Preferred repair:

- Keep the existing semaphore as the user-visible concurrency budget.
- Share a connected `tonic::transport::Channel` or a small channel pool.
- Construct or clone a request-local Flight client per admitted request.
- Do not hold a mutex across `do_exchange().await`.

Required proof:

- Add a focused concurrency test that starts a local Flight server and requires
  two admitted requests to reach the server concurrently before either response
  is released.
- The test should fail under serialized client execution and pass when the
  request path is genuinely concurrent.

Implementation checkpoint:

- Implemented on 2026-04-28 in
  `packages/rust/crates/xiuxian-wendao-runtime/src/transport/flight.rs`.
- The client now shares a lazily initialized tonic `Channel` instead of a
  mutex-protected `FlightClient`.
- Each admitted request builds a request-local Flight client and executes
  `do_exchange` outside the channel-initialization mutex.
- The regression proof is
  `flight_transport_client_runs_admitted_requests_without_client_lock_serialization`.

### P1: Gateway limits protect Studio HTTP routes but not necessarily Flight

`build_gateway_router` wraps `studio_routes()` with Tower load shedding,
timeout, and concurrency control before merging it into the root router. The
Flight service is mounted afterward as a separate route branch.

Impact:

- Studio HTTP routes have explicit overload behavior.
- The Flight branch may bypass the same gateway-level request budget.
- This violates the Tower audit principle that every branch in a hybrid service
  needs explicit readiness and backpressure semantics.

Preferred repair:

- Add Flight-specific gateway limits rather than assuming the Studio HTTP
  router layer applies to the mounted `any_service`.
- Keep Flight limits separately configurable from Studio HTTP limits because
  Flight payloads and streaming behavior have different cost profiles.

Required proof:

- Add a router-level test that validates Flight overload behavior through the
  mounted gateway path.
- Add a regression check that ordinary Studio HTTP overload handling remains
  unchanged.

Implementation checkpoint:

- Implemented on 2026-04-28 in
  `packages/rust/crates/xiuxian-wendao/src/bin/wendao/execute/gateway/command.rs`.
- The mounted Flight route now receives its own Tower layer with
  `load_shed`, `timeout`, and `concurrency_limit`.
- Flight budget knobs default to the Studio gateway budget for compatibility,
  but may be overridden through
  `XIUXIAN_WENDAO_GATEWAY_FLIGHT_CONCURRENCY_LIMIT` and
  `XIUXIAN_WENDAO_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS`.
- Existing Studio HTTP router budget semantics remain separate.

### P2: gRPC-Web and HTTP/1 compatibility should be deployment-scoped

`wendao_search_flight_server` and `wendao_search_flightsql_server` can enable
`accept_http1(true)` and `GrpcWebLayer` through deployment-scoped environment
flags, while the runtime-owned `wendao_flight_server` keeps the narrow
HTTP/2-only shape.

Impact:

- Browser-facing gRPC-Web compatibility is useful for same-origin frontend
  integration.
- Server-to-server Arrow Flight paths usually benefit from a narrower HTTP/2
  transport shape.
- Keeping compatibility layers implicit can make performance behavior harder to
  reason about.

Preferred repair:

- Make gRPC-Web and HTTP/1 compatibility an explicit host option.
- Default browser-facing gateway surfaces to compatibility mode only when that
  path is required.
- Keep pure runtime Flight hosts as narrow HTTP/2 transport surfaces unless a
  concrete consumer requires otherwise.

Implementation checkpoint:

- Implemented on 2026-04-28 for the gateway Flight mount and the standalone
  search Flight hosts.
- Compatibility is now opt-in so the default external edge remains HTTPS JSON
  and server-sent events.
- Gateway Flight compatibility can be enabled with
  `XIUXIAN_WENDAO_GATEWAY_FLIGHT_GRPC_WEB_ENABLED=true`.
- Standalone repo-search Flight compatibility can be enabled with
  `XIUXIAN_WENDAO_SEARCH_FLIGHT_GRPC_WEB_ENABLED=true`.
- Standalone FlightSQL compatibility can be enabled with
  `XIUXIAN_WENDAO_SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED=true`.
- By default, the corresponding server does not enable `accept_http1(true)` and
  does not wrap the service with `GrpcWebLayer`.
- The stable operational contract for these knobs is documented in
  [Gateway OpenAPI Contract Surface](../03_features/207_gateway_openapi_contract_surface.md#gateway-flight-runtime-controls).

### P2: Flight service construction wrappers are accumulating

The Studio Flight service has several construction helpers:

- default weights
- explicit weights
- current gateway state
- explicit project and config roots

Impact:

- The wrappers are readable in isolation, but parameter growth is already
  visible.
- Future transport options such as gRPC-Web mode, timeout policy, and branch
  concurrency would increase duplication.

Preferred repair:

- Introduce a small service-options record or builder for Studio Flight service
  construction.
- Keep public entrypoints narrow and compatibility-preserving.

### P3: HTTP API errors are repeatedly mapped into tonic statuses

Multiple Flight adapters repeat the same mapping from `StudioApiError` HTTP
status codes into `tonic::Status`.

Impact:

- The duplication is not a hot-path performance issue.
- It is a semantic drift risk as Flight endpoints grow.

Preferred repair:

- Move the mapping into one shared helper in the Studio Flight adapter layer.
- Keep endpoint-specific encoding logic local to each endpoint.

## Non-Goals

This audit does not require:

1. replacing Axum route mounting with a hand-written Hyper hybrid service
2. merging all standalone Flight binaries into the gateway
3. removing gRPC-Web support from browser-facing paths
4. changing Arrow Flight schema or route contracts
5. changing package ownership rules from
   `01_core/103_package_layering.md`

## Recommended Implementation Order

1. Fix the runtime Flight client lock so the configured in-flight budget can
   produce real concurrent transport requests.
2. Add explicit gateway-level Flight route overload controls.
3. Make gRPC-Web and HTTP/1 compatibility an explicit host option.
4. Collapse repeated Studio Flight service construction parameters into an
   options record.
5. Deduplicate `StudioApiError` to `tonic::Status` mapping.

## Verification Policy

Each implementation slice should include:

- one focused unit or integration test proving the changed transport behavior
- `cargo test` or `cargo nextest` scoped to the owning crate and test target
- `cargo fmt`
- `cargo clippy` only when the slice is ready to be considered fully landed

The first slice should target `xiuxian-wendao-runtime` because the mutex-held
`do_exchange().await` path is independent from gateway route composition and
directly affects transport throughput.
