# Feature: Dynamic SLUB Order and Per-Slab Object Offset Randomization

## Summary
Vary SLUB page order and the starting offset of objects within each new slab so cross-cache spraying and physical layout prediction become less deterministic. The mechanism remains independent of freelist randomization, AUTOSLAB type isolation, and critical-object jitter.

## Requirements
- REQ-1: Select slab page orders from a bounded per-cache sequence that preserves minimum object count, allocation constraints, NUMA behavior, and reclaimability.
- REQ-2: Choose an aligned random object-area offset for each slab while retaining metadata, redzones, guard behavior, and full object capacity accounting.
- REQ-3: Preserve SLUB fast paths, CPU partial lists, node lists, cache shrinking, memory hotplug, compaction, debug modes, memcg, and bulk allocation.
- REQ-4: Prevent deterministic boot- or cache-global sequences from being inferred through unprivileged allocator behavior, without claiming cryptographic secrecy.
- REQ-5: Coordinate with freelist randomization, random kmalloc caches, AUTOSLAB, KASAN, KFENCE, hardened usercopy, and slab merging rules.
- REQ-6: Bound fragmentation, high-order allocation failures, and memory waste with explicit fallback that retains offset randomization where safe.
- REQ-7: Measure heap-spray predictability, buddy fragmentation, allocation latency, memory pressure, boot, desktop, and server workload effects.

## Acceptance Criteria
- [ ] AC-1 [REQ-1]: Per-cache tests observe every configured legal order over repeated slab creation and never observe an order violating object-count or allocation constraints.
- [ ] AC-2 [REQ-2]: Object starts vary at the configured alignment across slabs, all objects remain inside their slab, and debug/redzone metadata remains valid.
- [ ] AC-3 [REQ-3]: SLUB selftests and stress cover CPU partial draining, node movement, shrink, hotplug, compaction, debug, memcg, and bulk paths without corruption or leaks.
- [ ] AC-4 [REQ-4]: Repeated boots and cache churn show no fixed public order/offset cycle, and status interfaces expose only aggregate enablement and failures.
- [ ] AC-5 [REQ-5]: The full matrix of freelist randomization, random kmalloc caches, AUTOSLAB, KASAN, KFENCE, hardened usercopy, and merging settings either passes or is rejected by explicit Kconfig dependencies.
- [ ] AC-6 [REQ-6]: Under induced fragmentation and pressure, allocation fallback follows the recorded bounded policy and never disables mandatory type isolation silently.
- [ ] AC-7 [REQ-7]: Automated attack-layout and performance workloads record confidence intervals, high-order failure rates, fragmentation, memory use, and accepted profile thresholds.

## Architecture
Implementation is localized to `../hdn-kernel/mm/slub.c`, internal structures in `../hdn-kernel/mm/slab.h`, common cache policy in `../hdn-kernel/mm/slab_common.c`, and configuration in `../hdn-kernel/mm/Kconfig`. It extends existing SLUB allocation decisions rather than adding a second slab allocator.

Each cache computes an allowed order set from object size, debug metadata, minimum objects, architecture limits, and administrator-independent HDN bounds. A per-boot keyed generator selects among those orders when allocating a new slab. After metadata placement, a second derived value selects an aligned leading offset that leaves all reported objects and metadata within the physical allocation.

Per-slab order and offset are stored in trusted slab metadata so free, validation, shrink, and diagnostic paths do not assume a cache-global layout. Fallback after a high-order failure steps through allowed lower orders and records a bounded event; it does not reuse an incompatible cache or turn off AUTOSLAB separation.

The verification matrix under `verification/configs/` exercises feature combinations and memory pressure. Tests add layout invariants and statistical checks to kernel selftests, while host KVM runs are required for allocator performance and realistic fragmentation measurements.

## Open Questions

None. The accepted defaults group dynamic order and per-slab offset as one allocator-layout mechanism and gate default use on measurement.

## Out of Scope
- Replacing `SLAB_FREELIST_RANDOM` or random kmalloc caches.
- Per-object payload jitter, which belongs to critical slab jitter.
- Cryptographic confidentiality of physical allocation layout.
- Supporting the legacy SLAB allocator in the first implementation.
