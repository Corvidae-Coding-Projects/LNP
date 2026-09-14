# Profile-scoped legacy executable emulation

## Goal
Provide narrow compatibility for legacy trampoline, signal-restorer, PLT, and dynamic-resolver patterns without granting broad writable-executable memory. Emulation is opt-in per authenticated application profile, preserves NX, and is invisible when the application does not require it.

## Requirements
- REQ-1: Define exact instruction and state-machine recognizers for supported trampoline, signal-restorer, PLT, and resolver patterns; reject partial, ambiguous, or self-modifying variants.
- REQ-2: Activate emulation only for an executable identity authorized by signed HDN policy, never through a process-controlled personality bit alone.
- REQ-3: Preserve non-executable stack and heap mappings and avoid creating writable-executable aliases while servicing a recognized legacy transition.
- REQ-4: Bind each emulated transition to the calling mapping, expected target, ABI, architecture mode, and immutable code bytes to prevent confused-deputy reuse.
- REQ-5: Rate-limit failures, expose actionable compatibility status through HDN desktop tooling, and keep raw register or address data out of unprivileged diagnostics.
- REQ-6: Test glibc and supported legacy runtimes, malformed sequences, races, ptrace, signals, seccomp, containers, and executable updates.
- REQ-7: Keep the mechanism absent from the default policy until a real supported workload requires it and the emulator passes the full adversarial corpus.

## How we will know it works
- [ ] AC-1 [REQ-1]: Byte-accurate positive fixtures for every supported pattern execute, while single-byte mutations, truncated sequences, alternate opcodes, and unexpected control states are rejected.
- [ ] AC-2 [REQ-2]: Unsigned, renamed, replaced, or policy-unlisted executables cannot activate emulation; a signed matching profile can activate only its declared pattern set.
- [ ] AC-3 [REQ-3]: `/proc` mapping inspection and page-table probes show no new W+X stack, heap, file, or alias mapping during successful emulation.
- [ ] AC-4 [REQ-4]: Race and replay tests cannot reuse a validated sequence after mapping mutation, target change, ABI switch, or executable identity update.
- [ ] AC-5 [REQ-5]: The security center reports a generic compatibility requirement and remediation path, while event records contain a stable pattern code without raw addresses.
- [ ] AC-6 [REQ-6]: Runtime fixtures cover native and compat modes where enabled, signals, ptrace, seccomp, namespaces, containers, updates, and concurrent mapping changes.
- [ ] AC-7 [REQ-7]: Default-policy tests prove all emulators are inactive, and enabling one named profile changes no unrelated process behavior.

## Technical plan
Recognition integrates with executable-memory fault handling and architecture signal paths under `../hdn-kernel/arch/x86/mm/` and `../hdn-kernel/arch/x86/kernel/signal.c`. Configuration and policy hooks reside in `../hdn-kernel/security/hardening/Kconfig` and `../hdn-kernel/security/hardening/core.c`; no legacy global sysctl is introduced.

Each emulator is a separate finite-state recognizer with a versioned pattern identifier. On an execute fault from NX memory, the kernel first authenticates the task's committed executable identity and signed profile, snapshots immutable instruction bytes with fault-safe reads, validates the complete pattern and register state, and performs only the semantic transition that the legacy sequence represents. It never makes the faulting page executable.

Resolver and PLT assistance binds the destination to the authenticated ELF mapping and loader state. Signal-restorer assistance accepts only kernel-established signal frames and expected ABI entry. Trampoline support restricts callable targets to the profile and mapping relationship established at load time. Any ambiguity follows the normal NX fault path.

Userspace compatibility policy is compiled by the existing tools in `../hdn-kernel/tools/hardening/` and surfaced by HDN OS without asking ordinary users to manage PaX flags. Tests live under `../hdn-kernel/tools/testing/selftests/hardening/`, with real application fixtures maintained in the distro integration matrix.

## Open questions

No open questions. The current plan uses signed profiles for each application and keeps the emulator off by default.

## Not included
- General binary translation or arbitrary instruction emulation.
- Granting RWX memory to preserve legacy behavior.
- Process-controlled global disable switches.
- Supporting an emulator without a named real-world compatibility requirement.
