# Feature: Isolated and Randomized BPF Executable Memory

## Summary
Give eBPF JIT images a dedicated, randomized executable address domain with strict write-then-execute transitions and no reuse as general module text. The design preserves verifier and JIT functionality while reducing cross-domain placement predictability and writable alias exposure.

## Requirements
- REQ-1: Allocate BPF JIT images from a dedicated randomized virtual region rather than the general module executable-memory range.
- REQ-2: Enforce non-executable writable construction followed by irreversible read-execute finalization for each image, with no simultaneous writable alias.
- REQ-3: Randomize region base, allocation placement, and safe intra-allocation image start without weakening alignment, branch reachability, or unwind metadata.
- REQ-4: Preserve JIT constant blinding, tail calls, trampolines, kfuncs, BPF-to-BPF calls, dispatcher generation, exception tables, and architecture-specific JIT invariants.
- REQ-5: Scrub freed image memory, delay predictable address reuse, and prevent a stale executable mapping from surviving program or trampoline destruction.
- REQ-6: Report only aggregate capacity and enforcement state to privileged HDN tooling; never expose randomization secrets or exact free-region layout.
- REQ-7: Validate verifier/JIT correctness and quantify memory, compilation, dispatch, networking, tracing, and allocation-fragmentation cost.

## Acceptance Criteria
- [ ] AC-1 [REQ-1]: Runtime page-table and allocator tests show every x86-64 BPF JIT image lies inside the dedicated region and no module text allocation uses it.
- [ ] AC-2 [REQ-2]: Construction-window tests can write but not execute, finalized images can execute but not write, and alias probes find no W+X physical page.
- [ ] AC-3 [REQ-3]: Repeated boots and allocation sequences demonstrate the configured entropy without violating x86 branch-distance or alignment constraints.
- [ ] AC-4 [REQ-4]: The upstream BPF selftest suite passes with JIT always-on, hardening level two, tail calls, trampolines, kfuncs, dispatchers, and exception paths enabled.
- [ ] AC-5 [REQ-5]: After program destruction, execution and reads through stale addresses fault, backing bytes are scrubbed before reuse, and grooming tests cannot force immediate deterministic placement.
- [ ] AC-6 [REQ-6]: `hdn-status` reports active, degraded, or exhausted state and aggregate use, while unprivileged interfaces reveal no new placement information.
- [ ] AC-7 [REQ-7]: BPF compilation and dispatch microbenchmarks plus networking and tracing workloads record confidence intervals and remain inside the accepted profile budget.

## Architecture
The generic allocation lifecycle begins in `../hdn-kernel/kernel/bpf/core.c` and `../hdn-kernel/include/linux/filter.h`. x86 executable range selection currently passes through `../hdn-kernel/arch/x86/mm/init.c` and generic `../hdn-kernel/mm/execmem.c`; the feature adds a BPF-specific execmem type and region descriptor instead of open-coding virtual allocation in the JIT.

At boot, KASLR-derived randomness selects a region base within architecture-safe bounds. The allocator maintains guard gaps and randomized free-range selection. Each image is allocated RW+NX, populated and validated, instruction/data caches are synchronized, then KERNEXEC finalizes it RX. Temporary writable reopening is forbidden; regeneration allocates a new image and atomically replaces the old reference.

Trampolines and dispatchers use subtypes with compatible reachability constraints but the same W^X lifecycle. Destruction first makes the image unreachable, waits for the appropriate RCU/task-trace grace period, removes execute permission, scrubs bytes, unmaps, and places the range into a bounded randomized reuse queue.

Configuration is added to `../hdn-kernel/security/hardening/Kconfig`; status flows through `../hdn-kernel/security/hardening/core.c`. Verification uses upstream tests under `../hdn-kernel/tools/testing/selftests/bpf/`, HDN W^X probes under `../hdn-kernel/tools/testing/selftests/hardening/`, and both TCG correctness and host-KVM performance stages.

## Open Questions

None. The accepted defaults select a dedicated x86-64 region, irreversible finalization, and measured default enablement.

## Out of Scope
- Permitting unprivileged BPF where the HDN policy otherwise disables it.
- Treating address randomization as a replacement for the verifier or JIT hardening.
- Sharing the BPF region with modules for space efficiency.
- Initial implementation of non-x86 BPF JIT regions.
