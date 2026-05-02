# xiuxian-git-repo

`xiuxian-git-repo` owns the reusable repository substrate that used to live
inside `xiuxian-wendao`.

Current slice scope:

- ghq-style managed mirror and checkout layout
- local checkout validation
- managed clone/fetch/checkout synchronization
- checkout lock lifecycle and stale-lock reclamation
- checkout metadata and probe-state observation
- backend-neutral public repo contracts and error taxonomy

Current implementation note:

- the public API is backend-neutral
- the crate no longer carries a runtime `git2` dependency
- repository open/probe/revision/drift logic is now backed by `gix`
- managed remote alignment is now persisted through `gix` local-config writes
  and repository reload
- managed remote target probing now uses `gix` remote ref-map inspection with
  explicit HEAD/branch/tag probe refspecs instead of `git ls-remote`
- managed bare clone, checkout clone, and origin fetch now also execute
  through `gix`, including mirror refspec parity for bare repos
- detached checkout now also executes through `gix` by combining index
  materialization, tracked-path pruning, worktree checkout, and detached HEAD
  reference mutation
- detached checkout now also refuses recursive directory removal when a stale
  tracked file path is unexpectedly backed by a directory, preserving
  unrelated untracked contents instead of deleting them during cleanup
- annotated tag probe results now resolve to the peeled target object so probe
  state stays comparable with checkout and tracking revisions
- local path remotes are normalized through canonical filesystem paths before
  drift comparison so tmp-backed mirrors and checkouts do not re-fetch solely
  due to path aliasing such as `/var` versus `/private/var`
- the touched materialization regressions now use tmp-backed cwd fixtures
  instead of operator-specific absolute paths
- the internal `gix` backend now lives under `src/backend/gix/` as a
  responsibility-sliced feature folder instead of one monolithic backend file
- project-policy gates now run through `rust-lang-project-harness` without
  disabled rules; public and branch modules carry intent docs for low-noise
  agent traversal
- remote clone/fetch/probe paths now also share
  `src/backend/gix/interrupt.rs`, which drives a bounded interrupt watchdog for
  managed remote operations and returns deterministic timeout errors through
  `XIUXIAN_GIT_REPO_REMOTE_OPERATION_TIMEOUT_SECS` (default `45`)
- the watchdog unit suite now also proves the fast success path returns well
  before the configured timeout budget, keeping this reliability guard
  performance-cheap for normal remote operations
- no production native `git` command bridge remains in `xiuxian-git-repo`
- `xiuxian-wendao` no longer carries `src/git/` compatibility models; callers
  now consume this crate through a minimal registered-repository adapter under
  `src/analyzers/repo_source.rs`
