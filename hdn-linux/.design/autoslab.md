# Automatic allocation-type slab isolation

## Goal
Automatically separate security-relevant allocation types and incompatible call-site domains so a freed object of one type is less useful for reclaiming as another. The design extends SLUB with compiler- or macro-provided allocation identity while retaining bounded memory overhead and explicit fallback behavior.

## Requirements
- REQ-1: Derive a stable allocation identity from complete C type information where available and from a reviewed call-site domain where APIs erase the type.
- REQ-2: Route eligible `kmalloc`-family allocations into isolated SLUB caches without changing caller-visible allocation, alignment, NUMA, accounting, or GFP semantics.
- REQ-3: Prevent cross-identity freelist reuse for designated security domains, including credentials, file objects, IPC, keyrings, namespace state, and policy objects.
- REQ-4: Handle modules, variable-size allocations, flexible arrays, memcg caches, bulk allocation, RCU-delayed free, and cache merging without silent loss of isolation.
- REQ-5: Bound cache proliferation and memory fragmentation through an explicit tiering policy that never merges mutually hostile security domains.
- REQ-6: Provide build-time coverage reporting and runtime status so missing type metadata is visible to developers but not disruptive to ordinary users.
- REQ-7: Measure allocator latency, memory overhead, boot time, desktop workload behavior, and allocation-heavy server workloads before profile promotion.

## How we will know it works
- [ ] AC-1 [REQ-1]: Compiler or macro tests assign identical stable identities to the same complete type across translation units and distinct identities to seeded incompatible types.
- [ ] AC-2 [REQ-2]: Allocation tests preserve size, alignment, NUMA node, memcg charge, GFP failure behavior, and sanitizer metadata for isolated allocations.
- [ ] AC-3 [REQ-3]: Freelist-reuse tests show that objects from designated hostile domains never occupy one another's isolated cache, including after CPU draining and memory pressure.
- [ ] AC-4 [REQ-4]: Module load/unload, variable and flexible-size objects, bulk paths, RCU frees, memcg creation/destruction, and `slab_nomerge` combinations pass without leaks or stale cache identities.
- [ ] AC-5 [REQ-5]: A stress workload remains below the recorded fragmentation and cache-count budget, and any fallback preserves mandatory security-domain separation.
- [ ] AC-6 [REQ-6]: The build report lists typed, call-site, and unclassified allocation sites; `hdn-status` reports aggregate coverage without exposing address or freelist information.
- [ ] AC-7 [REQ-7]: Repeated SLUB microbenchmarks, kernel builds, boot tests, desktop traces, networking, and filesystem workloads record confidence intervals and accepted costs.

## Technical plan
SLUB integration belongs in `../hdn-kernel/mm/slub.c`, `../hdn-kernel/mm/slab_common.c`, and `../hdn-kernel/include/linux/slab.h`. Existing `CONFIG_SLAB_BUCKETS` in `../hdn-kernel/mm/Kconfig` is treated as a useful primitive, not proof of general type isolation, because current substantive callers cover only a small part of the allocation surface.

Allocation identity is carried as compact compile-time metadata. Typed allocation macros derive an identity from the pointee type; untyped helpers use a stable domain token attached to the call site or wrapper API. AUTOTYPENAME-style metadata must be deterministic for a source build and independent of randomized virtual addresses. AUTOSTACK-style isolation is limited to heap objects whose allocating API can preserve identity; literal stack variables are outside SLUB.

The allocator maps identities to cache classes during initialization. Mandatory security domains receive dedicated caches. Lower-risk identities may share a bucket only when their size, constructor, destructor, RCU behavior, usercopy region, memcg behavior, and threat class are compatible. Cache merging cannot cross mandatory boundaries, and fallback under resource pressure is a controlled allocation failure rather than hostile merging.

Modules register identity metadata before executable finalization and cannot choose an existing privileged domain arbitrarily. Runtime counters and bounded events are implemented through `../hdn-kernel/security/hardening/core.c`; raw freelist state remains privileged. KASAN, KFENCE, KMSAN, hardened usercopy, and slab debugging each receive a matrix configuration in `verification/configs/`.

## Open questions

No open questions. The current plan uses type-aware isolation, measured tiers, and x86-64 release testing.

## Not included
- One cache for every source allocation expression regardless of cost.
- Treating current `CONFIG_SLAB_BUCKETS` coverage as complete AUTOSLAB.
- Hiding allocation failures or silently merging mandatory hostile domains.
- Redesigning non-SLUB allocators in the first implementation.
