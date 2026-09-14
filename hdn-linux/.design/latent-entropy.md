# Conservative latent entropy mixing

## Goal
Evaluate compiler-generated and runtime execution variability as supplemental random-pool input for entropy-starved systems, while assigning it zero entropy credit. The mechanism may improve state diversity but can never make readiness claims, unblock cryptographic consumers, or replace hardware and operating-system entropy sources.

## Requirements
- REQ-1: Define an explicit threat model limited to supplemental state mixing on systems with sparse early-boot events, excluding adversaries that can fully observe or control execution state.
- REQ-2: Instrument only reviewed low-frequency boot, fork, and interrupt paths with a bounded mixing function that does not expose collected state or compiler-generated constants.
- REQ-3: Feed data into the Linux random subsystem without entropy credit and without advancing CRNG readiness.
- REQ-4: Preserve deterministic test and crash-reproduction modes through a test-only seed path that cannot be enabled by production policy.
- REQ-5: Coexist with hardware RNGs, jitter entropy, device randomness, virtual machines, hibernation, cloning, and early userspace without double-counting correlated input.
- REQ-6: Validate compiler stability, undefined-behavior absence, stack initialization, sanitizer compatibility, and data-erasure behavior for instrumented state.
- REQ-7: Measure boot, fork, interrupt, power, and code-size overhead and keep the feature profile-gated unless a supported entropy-starved platform demonstrates value.

## How we will know it works
- [ ] AC-1 [REQ-1]: The security analysis states what uncertainty may be added, what an observer can predict, and why no cryptographic entropy claim derives from the feature.
- [ ] AC-2 [REQ-2]: Binary inspection confirms instrumentation only at the reviewed site list, bounded stack/register use, and no export of internal state through logs or status APIs.
- [ ] AC-3 [REQ-3]: Random-subsystem tests prove the input path adds zero credited bits and cannot change the transition to initialized by itself.
- [ ] AC-4 [REQ-4]: Deterministic kernel tests reproduce identical instrumented traces with the test seed, while production builds reject the test interface and symbol.
- [ ] AC-5 [REQ-5]: Boot tests cover hardware RNG present/absent, virtio RNG, jitter entropy, VM clone, hibernation resume, and early consumers without duplicate-credit or readiness regressions.
- [ ] AC-6 [REQ-6]: GCC builds plus KASAN, KCSAN, KMSAN, UBSAN-supported checks, stack-init tests, and compiler differential tests report no uninitialized read or undefined behavior.
- [ ] AC-7 [REQ-7]: Repeated measurements report boot, fork, interrupt, power where measurable, and text-size overhead; the default profile remains off absent platform-specific acceptance evidence.

## Technical plan
Integration uses the existing random subsystem in `../hdn-kernel/drivers/char/random.c` and compiler support under `../hdn-kernel/scripts/gcc-plugins/`. Configuration and honest status reside in `../hdn-kernel/security/hardening/Kconfig` and `../hdn-kernel/security/hardening/core.c`.

The compiler pass creates per-site mixing state from defined integer operations, initialized values, timing samples already legal at that phase, and domain-separated compile-time constants. It never reads padding or uninitialized memory. Sites are selected from an allowlist based on frequency and platform need, not inserted indiscriminately into hot paths.

Runtime contributions call a dedicated no-credit mixing wrapper that is mechanically prevented from invoking entropy-credit functions. Resume and VM-clone paths reseed domain state from already available pool state and device events but do not claim new entropy. Production status reports active mixing and zero credited bits.

Verification includes symbol and call-graph assertions that the wrapper cannot reach crediting APIs, compiler fixtures, deterministic test builds, and boot matrices from `verification/bin/hdn-verify`. Statistical output is used only to detect broken or constant mixing, never to estimate security bits.

## Open questions

No open questions. The current plan treats latent entropy as an optional extra, gives it no security credit, and controls it by profile.

## Not included
- Claiming a quantified number of cryptographic entropy bits.
- Unblocking `/dev/random` or CRNG initialization.
- Replacing hardware RNG, jitter entropy, or device-event collection.
- Default enablement without a demonstrated entropy-starved target.
