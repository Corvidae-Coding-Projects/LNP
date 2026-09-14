# Systematic reference-count hardening

## Goal
Move lifetime-controlling counters onto Linux's saturating `refcount_t` semantics and continuously detect regressions back to unchecked atomics. The design prioritizes semantic conversion and proof of ownership behavior over a blanket textual replacement.

## Requirements
- REQ-1: Inventory `atomic_t`, `atomic_long_t`, and plain integer fields used primarily to control object lifetime, organized by reachable attack surface and subsystem ownership.
- REQ-2: Convert eligible counters to the API in `include/linux/refcount.h`, preserving memory ordering, zero-to-one initialization rules, and object release behavior.
- REQ-3: Use `refcount_inc_not_zero()`, `refcount_dec_and_test()`, and checked acquisition patterns so saturation prevents use-after-free without creating resurrection paths.
- REQ-4: Keep counters with arithmetic, statistics, resource limits, or mixed semantics out of automatic conversion and document them in a machine-checkable exception file.
- REQ-5: Add Coccinelle and static-analysis gates that detect newly introduced atomic-as-refcounter patterns and suspicious unchecked return values.
- REQ-6: Validate high-risk conversions with concurrency stress, KCSAN, fault injection, and subsystem selftests before enabling any stricter runtime response.

## How we will know it works
- [ ] AC-1 [REQ-1]: A generated inventory names every candidate field, definition path, mutation sites, release site, user reachability, and disposition, with no unreviewed high-risk candidate remaining.
- [ ] AC-2 [REQ-2]: Converted objects pass their subsystem tests and retain the intended acquire/release ordering under architecture litmus tests and code review.
- [ ] AC-3 [REQ-3]: Underflow, overflow, increment-from-zero, and saturation test cases produce the documented safe behavior and never free a saturated live object.
- [ ] AC-4 [REQ-4]: Static analysis excludes documented mixed-purpose counters and fails if an exception lacks a source path, field name, semantic reason, and reviewer-visible expiry condition.
- [ ] AC-5 [REQ-5]: `scripts/coccinelle/api/atomic_as_refcounter.cocci` plus HDN-specific rules run in CI and reject a seeded atomic lifetime counter and ignored checked-acquire result.
- [ ] AC-6 [REQ-6]: KCSAN, refcount fault injection, syzkaller lifetime programs, and converted-subsystem suites complete without new races, leaks, premature frees, or warnings.

## Technical plan
The implementation builds on `../hdn-kernel/include/linux/refcount.h` and `../hdn-kernel/lib/refcount.c`; it does not create a competing HDN counter type. The existing semantic patch at `../hdn-kernel/scripts/coccinelle/api/atomic_as_refcounter.cocci` becomes the seed for a versioned inventory and prevention gate stored with the verification tooling.

Conversion is deliberately subsystem-by-subsystem. Each candidate record states whether the count owns lifetime, whether zero is terminal, whether increments may race with final release, and which memory-order guarantee consumers require. Only pure lifetime counters are mechanically converted. Mixed counters require a prior structural split into ownership and accounting fields.

Runtime response uses upstream saturation semantics. HDN policy may choose bounded reporting or panic for kernel self-protection testing, but production cannot restore wrapping or permit decrement from saturation to free an object. Events emitted through `../hdn-kernel/security/hardening/core.c` contain the warning class and build-relative location, not object addresses.

The verification environment adds targeted Coccinelle runs, KCSAN profiles, and syzkaller descriptions for converted high-risk entry points. Results are tied to the exact source commit and stored outside Git as test artifacts, with concise summaries under the release manifest.

## Open questions

No open questions. The current plan converts counters in reviewed stages and keeps upstream saturation behavior.

## Not included
- Converting counters that measure statistics, quotas, sequence numbers, or resource availability.
- Introducing a new compiler language extension for reference counts.
- Treating all `atomic_t` uses as bugs.
- Weakening saturation to preserve buggy wraparound behavior.
