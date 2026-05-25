# xiuxian-wendao-server

`xiuxian-wendao-server` is the narrow Flight and gRPC transport boundary for
Wendao.

This crate does not own Studio HTTP routes, OpenAPI contracts, repository
analysis, parsers, config loading, DuckDB access, git repository management, or
frontend-facing state. Those heavier gateway concerns live in
`xiuxian-wendao-studio`.

## Features

- `transport`: exposes the runtime-owned Flight/gRPC route contracts, route
  provider traits, request validators, and `WendaoFlightService` facade.

## Dataset Ontology Handoff

The transport feature exposes `/ontology/dataset/materialize` as the
dataset-to-ontology handoff route used by the Gateway-facing Flight service.
The route carries only the admitted mapping manifest in Flight metadata:
`x-wendao-dataset-ontology-contract-id`,
`x-wendao-dataset-ontology-mapping-id`, and
`x-wendao-dataset-ontology-manifest`.

This crate owns the stable route constants, metadata validation, and provider
trait seam. It does not execute DuckDB, parse raw CSV, mutate RDF, or read
project configuration. A host such as `xiuxian-wendao-studio` must attach a
`DatasetOntologyMaterializeFlightRouteProvider` that materializes already
admitted read-model batches through the runtime-owned SQL/Arrow substrate.

## Ontology Candidate Inspection Handoff

The transport feature also exposes `/ontology/candidates/inspect` as the
candidate read-model inspection route. The request is admitted through Flight
metadata in `x-wendao-ontology-candidate-inspection` and carries the compact
JSON fields `schemaVersion`, `epistemeRegistryId`, and `runId`.

This crate owns the route constant, request metadata validation, cache-key
derivation, and `OntologyCandidateInspectionFlightRouteProvider` trait. It does
not resolve `wendao.toml`, load private Episteme repositories, inspect DuckDB,
read candidate TSV projections, mutate RDF, or parse raw source files. A host
such as `xiuxian-wendao-studio` must attach the provider that resolves registry
configuration and returns Arrow inspection batches.

The default feature set is empty so downstream crates do not accidentally pull
transport or domain dependencies.

## Dependency Boundary

The crate's direct dependency set is intentionally limited to Arrow Flight,
Arrow schema/array types, Tokio streams, Futures, Base64, Async Trait, and
Tonic. A full dependency tree can still show `axum` because `arrow-flight`
depends on Tonic's generated Flight service support, and Tonic currently routes
that support through its internal router feature. That transitive implementation
detail must not become a direct `xiuxian-wendao-server` dependency or a reason to
move Studio HTTP ownership back into this crate.
