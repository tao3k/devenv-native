<!-- ASP-HOOK-TRIGGER-PROMPT:MANAGED-BEGIN -->

ASP hook blocked `{reason}`; do not retry raw read/search commands on the same source.
Codex: if an ASP search agent thread is already open for this main task, call `send_input` on that thread with the safe route below and wait for `asp-search-subagent(role,action,evidence,missing,next,risk)`.
Otherwise call `spawn_agent` with `fork_turns="none"` and a self-contained ASP Explorer branch prompt; keep model and reasoning settings in Codex config.
If subagents are unavailable, run the safe route directly.

{routes}

<!-- ASP-HOOK-TRIGGER-PROMPT:MANAGED-END -->

<!-- ASP-HOOK-TRIGGER-PROMPT:USER-EXTENSIONS-BEGIN -->
<!-- Add project-local hook trigger guidance below. `asp hook install` preserves this block. -->
<!-- ASP-HOOK-TRIGGER-PROMPT:USER-EXTENSIONS-END -->
