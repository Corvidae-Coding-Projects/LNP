# Compiler-guided Spectre dataflow hardening

## Goal
Add Respectre-style analysis that identifies attacker-influenced control and data flows vulnerable to speculative execution and inserts the narrowest correct masking or barrier primitive. The feature complements hardware and entry-path mitigations by finding source-level Spectre-v1 and selected Spectre-v4 gadgets that manual annotations miss.

## Requirements
- REQ-1: Define taint sources for syscall arguments, user copies, packet data, device input, BPF-controlled values, and other attacker-influenced kernel data.
- REQ-2: Detect bounds-check bypass patterns where a tainted condition controls an array index, pointer selection, length, function target, or disclosure-capable load.
- REQ-3: Insert or require architecture-approved masking and speculation barriers from `include/linux/nospec.h` and x86 barrier primitives, choosing data dependency over a full barrier when sound.
- REQ-4: Detect selected speculative-store-bypass hazards where an attacker-influenced store can be bypassed before a security-sensitive load, and coordinate with SSBD state.
- REQ-5: Maintain a reviewed annotation and suppression vocabulary for sanitizers, constant-time code, proven-safe dependencies, and unavoidable assembly boundaries.
- REQ-6: Preserve optimizer correctness and support the release GCC build, while reporting an explicit unsupported state for compiler configurations without equivalent analysis.
- REQ-7: Measure gadget reduction and workload cost across syscalls, networking, storage, BPF, browsers, and kernel compilation before default promotion.

## How we will know it works
- [ ] AC-1 [REQ-1]: Analyzer fixtures show taint propagation through arithmetic, structures, aliases, calls, and usercopy boundaries, and do not treat trusted kernel constants as tainted.
- [ ] AC-2 [REQ-2]: A curated gadget corpus and seeded kernel examples are detected with source locations, while reviewed safe patterns remain below the recorded false-positive budget.
- [ ] AC-3 [REQ-3]: Generated machine code contains the intended mask or barrier, and litmus plus side-channel tests show the protected gadget no longer transmits the secret.
- [ ] AC-4 [REQ-4]: Spectre-v4 fixtures are either instrumented or proven protected by active SSBD; disabling SSBD cannot silently remove software coverage for a required site.
- [ ] AC-5 [REQ-5]: CI rejects a suppression lacking category, source span, justification, owner, and invalidation condition, and rejects suppressions that outlive the referenced code.
- [ ] AC-6 [REQ-6]: GCC builds pass full verification; Clang builds either run a coverage-equivalent pass or fail the HDN option at configuration time.
- [ ] AC-7 [REQ-7]: Automated scans report residual gadgets, and repeated performance runs record confidence intervals for the specified workloads and each instrumentation class.

## Technical plan
The feature consumes the canonical helpers in `../hdn-kernel/include/linux/nospec.h`, architecture barriers in `../hdn-kernel/arch/x86/include/asm/barrier.h`, and mitigation state managed under `../hdn-kernel/arch/x86/kernel/cpu/bugs.c`. Configuration and truthful status reporting are added to `../hdn-kernel/security/hardening/Kconfig` and `../hdn-kernel/security/hardening/core.c`.

The initial analyzer is a GCC interprocedural pass integrated through `../hdn-kernel/scripts/gcc-plugins/`. It models attacker sources, validation predicates, speculation-insensitive dependencies, and disclosure sinks. Findings have stable identifiers derived from semantic source locations. For patterns that can be safely transformed, the pass emits a mask or dependency; ambiguous cases fail the hardened build until source is annotated or rewritten.

Spectre-v1 handling prefers `array_index_nospec()`-style masking because it localizes cost. Barriers are used for pointer selection, complex control flow, and cases where masking cannot express the invariant. Spectre-v4 analysis considers store/load aliasing and active CPU mitigation state, but never assumes a boot parameter is present without querying the compiled mitigation contract.

The suppression file is versioned with the verification tools and checked for stale entries. Machine-code auditing and side-channel fixtures run from `verification/bin/hdn-verify`; runtime tests live in `../hdn-kernel/tools/testing/selftests/hardening/`. TCG validates functional instrumentation while host hardware is mandatory for timing and leakage measurements.

## Open questions

No open questions. The current plan starts with GCC on x86-64, keeps suppressions visible in source, and requires hardware measurements before default rollout.

## Not included
- Claiming complete protection against every transient-execution class.
- Replacing CPU microcode or architecture-level mitigations.
- Hiding unresolved analyzer findings behind a global warning-only mode.
- Using TCG results as evidence of side-channel resistance.
