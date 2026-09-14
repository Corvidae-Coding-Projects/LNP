# Feature: RAP-Style Forward and Return Control-Flow Integrity

## Summary
Add a coherent HDN control-flow integrity design that covers indirect-call type integrity and return-address integrity rather than treating upstream forward-edge CFI as complete. The initial implementation targets the GCC-built x86-64 release kernel, with objtool verification and an explicit Clang compatibility path.

## Requirements
- REQ-1: Protect indirect calls and jumps with type-compatible target validation across the built-in kernel and loadable modules.
- REQ-2: Protect return addresses against stack overwrite and replay, including interrupt, exception, syscall, context-switch, and signal-adjacent kernel entry paths.
- REQ-3: Support an optional RAP-XOR-style encoding layer only when it adds independently tested resistance and does not become a substitute for target validation.
- REQ-4: Reject modules whose CFI metadata, compiler identity, or type-hash scheme is incompatible with the running kernel.
- REQ-5: Preserve ftrace, kprobes, static calls, alternatives, BPF trampolines, livepatch where enabled, unwinding, crash dumps, and sanitizers through explicit adapters.
- REQ-6: Make violations deterministic, fail closed before the corrupted transfer, and expose only bounded build-relative diagnostic data.
- REQ-7: Quantify code-size, kernel-build, syscall, networking, and context-switch costs before enabling each layer in the default profile.

## Acceptance Criteria
- [ ] AC-1 [REQ-1]: Negative tests replace an indirect target with a type-incompatible function and are stopped before invocation; positive tests cover legitimate callbacks and module interfaces.
- [ ] AC-2 [REQ-2]: Corrupted, copied, and replayed return addresses are rejected across normal calls, interrupts, exceptions, syscalls, and task switches, while ORC unwinding remains correct.
- [ ] AC-3 [REQ-3]: Enabling the XOR layer changes emitted return sequences and defeats the dedicated replay corpus; disabling it leaves the base return-integrity mechanism fully effective.
- [ ] AC-4 [REQ-4]: A module built with missing, mismatched, or mutated CFI metadata is rejected before relocation finalization, while a matching signed module loads and unloads successfully.
- [ ] AC-5 [REQ-5]: Existing selftests for tracing, probes, static calls, BPF, livepatch where configured, unwinding, kdump, KASAN, KCSAN, and KMSAN pass in their supported matrices.
- [ ] AC-6 [REQ-6]: Every injected violation produces a stable HDN event and controlled termination or panic according to sealed policy, without disclosing raw randomized addresses.
- [ ] AC-7 [REQ-7]: Repeated benchmark runs report confidence intervals for text growth, build time, syscall latency, hackbench, networking throughput, and kernel compilation; default promotion records the accepted cost.

## Architecture
Upstream forward-edge CFI configuration is defined in `../hdn-kernel/arch/Kconfig`, but its scope does not provide return integrity. HDN adds orthogonal configuration in `../hdn-kernel/security/hardening/Kconfig`. The initial compiler instrumentation uses the existing GCC plugin framework under `../hdn-kernel/scripts/gcc-plugins/`, with a companion metadata format consumed by module loading and an objtool pass under `../hdn-kernel/tools/objtool/` that verifies coverage after compilation.

Forward-edge metadata assigns normalized type identities to address-taken functions and call sites. Hashes are domain-separated by build identity and representation version; they are not relied upon as secrets. Direct calls remain unchanged. Assembly and dynamically generated call paths must use explicit annotations that objtool validates, so unsupported naked edges cannot silently escape coverage.

Return integrity uses a protected per-task context initialized during task creation and switched alongside architecture thread state. Instrumented prologues bind the expected return site to the protected context; epilogues authenticate it before control transfer. Interrupt and exception assembly update a separate nested domain so asynchronous entry cannot consume normal-call state. The optional XOR encoding is layered over this authenticated state and uses a boot secret without weakening crash handling or reproducible source builds.

Module finalization in `../hdn-kernel/kernel/module/main.c` validates metadata before executable permission is granted. Dynamic facilities receive narrow adapters, and `../hdn-kernel/kernel/module/strict_rwx.c` remains responsible for final W^X. Failure telemetry flows through `../hdn-kernel/security/hardening/core.c`; raw secrets, hashes useful as bypass oracles, and virtual addresses never cross the UAPI.

Verification combines compiler unit fixtures, objtool coverage checks, binary disassembly assertions, LKDTM-style corruption tests, and the QEMU hardening suite in `../hdn-kernel/tools/testing/selftests/hardening/`. GCC is the release gate; Clang builds must either use an equivalently verified implementation or fail configuration rather than silently omitting the feature.

## Open Questions

None. The accepted defaults select GCC/x86-64 as the first release target, require truthful Clang handling, and gate default enablement on measured compatibility and cost.

## Out of Scope
- Claiming that forward-edge Clang CFI alone provides RAP-equivalent coverage.
- Supporting out-of-tree binary modules without matching metadata.
- Using secret type hashes as the sole protection.
- Cross-architecture return instrumentation in the first implementation.
