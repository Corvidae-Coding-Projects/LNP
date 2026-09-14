# Feature: Lifecycle-Sealed Dynamic Kernel Objects

## Summary
Generalize HDN's existing policy and mitigation-table sealing into a typed lifecycle for dynamically allocated kernel objects that become immutable after configuration. KERNSEAL protects both logical mutation paths and writable memory aliases, with narrowly defined replacement rather than reopening sealed objects.

## Requirements
- REQ-1: Provide a typed API for construct, populate, validate, seal, read, replace, and destroy phases of eligible dynamic objects.
- REQ-2: Ensure sealed object storage has no writable kernel alias and shares no protection granule with unrelated mutable data.
- REQ-3: Require replacement-by-copy for supported updates: construct a new object, validate it, atomically publish it, then retire the old sealed instance safely.
- REQ-4: Support RCU readers, reference-counted readers, NUMA placement, suspend/resume, memory pressure, and teardown without temporarily unsealing published objects.
- REQ-5: Integrate signed HDN policy, mitigation state, security dispatch data, and future eligible registries using per-type seal descriptors rather than raw address registration.
- REQ-6: Reject post-seal mutation, invalid lifecycle transitions, type confusion, range overlap, and unregistered writable aliases with a bounded deterministic response.
- REQ-7: Quantify memory fragmentation, TLB cost, replacement latency, and read-path overhead before enabling each object class by default.

## Acceptance Criteria
- [ ] AC-1 [REQ-1]: API unit tests exercise every legal transition and reject populate-after-seal, double seal, publish-before-validation, and destroy-while-published operations.
- [ ] AC-2 [REQ-2]: Page-table inspection and negative writes prove sealed pages are read-only through all kernel aliases, and allocator tests prove no mutable object shares the protected granule.
- [ ] AC-3 [REQ-3]: Concurrent replacement tests show readers observe either the complete old object or complete new object, never partial state, and the old object is reclaimed only after its reader discipline completes.
- [ ] AC-4 [REQ-4]: RCU, refcount, NUMA, suspend/resume, memory pressure, and teardown stress runs complete without write windows, leaks, use-after-free, or stale mappings.
- [ ] AC-5 [REQ-5]: Existing HDN policy and mitigation-table sealing migrate to the typed API and retain all current policy signature, generation, commit, and runtime-seal tests.
- [ ] AC-6 [REQ-6]: Seeded mutation, overlap, wrong-type, and alias attacks are rejected before state changes and emit only stable build-relative diagnostics.
- [ ] AC-7 [REQ-7]: Microbenchmarks report allocation waste, protected-page count, TLB effects, replacement latency, and read throughput for every default object class.

## Architecture
The current seed implementation is in `../hdn-kernel/security/hardening/core.c`, where the HDN policy and mitigation table already become immutable. KERNSEAL extracts a generic internal service while keeping public policy operations in their existing module. Configuration remains in `../hdn-kernel/security/hardening/Kconfig`.

Each sealable type defines a descriptor with size and alignment constraints, validation callback, reader discipline, allowed replacement phase, and event class. Callers receive typed opaque handles rather than arbitrary address-plus-length registration. Storage comes from dedicated page-granular pools or seal-aware caches so page permission changes never freeze unrelated objects.

Sealing first validates contents, flushes pending initialization writes, removes writable aliases through KERNEXEC primitives, then publishes the object with release ordering. Updates allocate a fresh writable candidate and atomically replace the published pointer. RCU or reference-count completion retires the old read-only object; the implementation remaps it only for allocator-internal destruction after it is unreachable.

The API is internal and not exported indiscriminately to modules. Approved in-tree consumers are enumerated at build time. Runtime state is summarized through the existing UAPI in `../hdn-kernel/include/uapi/linux/hardening.h`, but object addresses, contents, and protection keys are never exposed.

Verification lives under `../hdn-kernel/tools/testing/selftests/hardening/` with fault-injection helpers for illegal writes, stale handles, concurrent replacement, and pressure. KERNEXEC tests prove physical alias closure; KASAN, KCSAN, and lockdep cover lifetime and ordering.

## Open Questions

None. The accepted defaults select replacement-by-copy, typed registration, and measured per-class rollout.

## Out of Scope
- Arbitrary page sealing requested by loadable modules.
- Reopening a published object for in-place editing.
- Sealing mutable counters merely because they are security related.
- Userspace access to sealed kernel object contents.
