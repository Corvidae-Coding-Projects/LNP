# Critical slab object jitter and reuse resistance

## Goal
Make heap grooming against a small set of critical kernel objects less deterministic by varying payload placement, checking hidden canaries, and reducing immediate same-address reuse. This is a targeted defense for credentials, file state, and similarly high-value objects, not a claim that random padding repairs memory-safety bugs.

## Requirements
- REQ-1: Define a reviewed list of critical object classes and the exploit primitives that placement jitter, canaries, and delayed reuse are intended to disrupt.
- REQ-2: Randomize the offset of the usable payload within allocator-owned storage while preserving required alignment, constructor semantics, RCU behavior, usercopy bounds, and sanitizer metadata.
- REQ-3: Store and validate a keyed per-object canary outside attacker-controlled payload fields, checking it before security-sensitive use and at free.
- REQ-4: Prevent immediate deterministic reuse of a freed critical slot through a bounded per-cache quarantine or randomized reuse policy compatible with memory pressure.
- REQ-5: Keep pointer translation internal to typed allocation/free helpers so generic `kfree()` cannot silently mis-handle a jittered object.
- REQ-6: Preserve crash-dump analysis, slab debugging, KASAN, KFENCE, KMSAN, lockdep, memcg accounting, NUMA, and RCU diagnostics.
- REQ-7: Measure memory overhead, allocator latency, cache footprint, and attack-corpus success probability for each protected class.

## How we will know it works
- [ ] AC-1 [REQ-1]: The design inventory names every enabled object class, allocating and freeing APIs, attacker influence, intended disruption, and explicit residual risk.
- [ ] AC-2 [REQ-2]: Repeated allocation samples demonstrate the configured aligned offset distribution, while object constructors, usercopy checks, RCU callbacks, and sanitizers pass.
- [ ] AC-3 [REQ-3]: Seeded underflow, overflow, stale-write, and wrong-type-free tests corrupt the canary and are detected before object reuse or security-sensitive consumption.
- [ ] AC-4 [REQ-4]: Grooming tests show freed critical slots are not predictably returned on the next compatible allocation, including across CPU freelists; pressure fallback remains bounded and observable.
- [ ] AC-5 [REQ-5]: Compile-time types or wrapper checks reject freeing a protected payload through the generic path, and no raw backing pointer escapes its allocator module.
- [ ] AC-6 [REQ-6]: Slab debug, KASAN, KFENCE, KMSAN, lockdep, memcg, NUMA, RCU, and crash-dump test configurations retain correct object attribution.
- [ ] AC-7 [REQ-7]: Per-class benchmarks and an automated heap-grooming corpus record overhead and predictability reduction; only classes within the accepted profile budget are default-on.

## Technical plan
Allocator support is implemented in `../hdn-kernel/mm/slub.c`, `../hdn-kernel/mm/slab_common.c`, and `../hdn-kernel/include/linux/slab.h`. Protected types use explicit typed helpers located with their owning subsystem; the first candidates are selected from the credential and file-object allocation paths after full call-site inventory.

A protected cache allocates a fixed maximum envelope sized for alignment, jitter range, canary, and allocator metadata. A boot-secret pseudorandom function chooses an aligned payload offset from the allowed range for each allocation. Translation metadata is stored in allocator-controlled bytes validated before use; it is not stored in a predictable writable field inside the payload. The returned payload pointer remains stable for the object's lifetime.

The canary binds cache identity, backing slot, payload offset, and allocation generation to a boot secret. It is checked at typed lookup boundaries and free. The reuse layer holds a bounded number of retired slots per cache and CPU, drains unpredictably, and has explicit pressure behavior. It never bypasses RCU grace periods or memory-cgroup accounting.

AUTOSLAB ensures hostile types do not share the cache, while this feature adds within-type placement and temporal uncertainty. Events use `../hdn-kernel/security/hardening/core.c` and redact allocator secrets and addresses. Tests under `../hdn-kernel/tools/testing/selftests/hardening/` include statistical distribution checks, corruption probes, and deterministic seeded mode available only in test kernels.

## Open questions

No open questions. The current plan starts with a small reviewed set of critical objects and expands it only after benchmarks.

## Not included
- Applying maximum jitter and quarantine to every kernel allocation.
- Treating randomness as a replacement for fixing use-after-free or overflow defects.
- Exposing offsets, canaries, or quarantine state to unprivileged users.
- Supporting allocators other than SLUB in the first implementation.
