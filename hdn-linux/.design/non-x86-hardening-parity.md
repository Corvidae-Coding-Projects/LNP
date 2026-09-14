# Feature: Non-x86 Hardening Parity Framework

## Summary
Define how HDN expands beyond x86-64 without overstating equivalent protection on architectures with different page-table, control-flow, uaccess, and executable-memory primitives. The first deliverable is a capability contract and conformance suite for arm64 and RISC-V implementations.

## Requirements
- REQ-1: Define architecture-neutral security invariants for KERNEXEC, UDEREF, RAP, KERNSEAL, private stacks, BPF execmem, and mitigation status.
- REQ-2: Provide architecture capability descriptors that report enforced, hardware-backed, software-backed, degraded, unsupported, and not-applicable states per subfeature.
- REQ-3: Keep architecture mechanics under `arch/` and shared policy/lifecycle logic under `security/hardening/`, with no x86 constants in generic UAPI.
- REQ-4: Establish arm64 and RISC-V bring-up sequences that map native primitives to each invariant or reject configuration honestly.
- REQ-5: Build and boot architecture-specific hardening configs under emulation, then require representative hardware for performance, side-channel, firmware, and interrupt validation.
- REQ-6: Prevent release tooling and `hdn-status` from collapsing partial architecture coverage into one misleading enabled bit.
- REQ-7: Preserve the x86-64 default profile while ports are incomplete and prevent generic refactors from weakening its verified behavior.

## Acceptance Criteria
- [ ] AC-1 [REQ-1]: A versioned invariant table names the protected asset, attacker capability, required property, lifecycle, and architecture test for every shared mechanism.
- [ ] AC-2 [REQ-2]: Unit tests enumerate every descriptor state, and a build fails if a selected feature lacks an explicit architecture implementation or unsupported declaration.
- [ ] AC-3 [REQ-3]: Static checks find no x86-only constants or structures in `include/uapi/linux/hardening.h` or generic policy code, and architecture hooks have bounded typed interfaces.
- [ ] AC-4 [REQ-4]: arm64 and RISC-V implementation plans identify native execute permissions, uaccess controls, CFI/return protection, TLB behavior, stack mapping, and JIT constraints with no false equivalence claims.
- [ ] AC-5 [REQ-5]: Cross-build and QEMU boot matrices pass for arm64 and RISC-V; hardware-only gates are named, scripted, and remain visibly unqualified until executed.
- [ ] AC-6 [REQ-6]: `hdn-status` and release manifests show per-subfeature coverage and refuse a global protected claim when any mandatory invariant is degraded.
- [ ] AC-7 [REQ-7]: The full x86-64 regression suite passes unchanged after generic hook introduction, with binary and runtime invariant comparisons recorded.

## Architecture
The generic contract lives beside current HDN policy in `../hdn-kernel/security/hardening/core.c` and its UAPI in `../hdn-kernel/include/uapi/linux/hardening.h`. Configuration dependencies extend `../hdn-kernel/security/hardening/Kconfig`. Architecture implementations reside in `../hdn-kernel/arch/x86/`, `../hdn-kernel/arch/arm64/`, and `../hdn-kernel/arch/riscv/` behind typed operations tables initialized before policy sealing.

The contract is invariant-based rather than implementation-name-based. For example, UDEREF requires accidental kernel user-memory access to fail outside a declared window; x86 SMAP and arm64 PAN are implementations with distinct status. KERNEXEC requires alias-safe W^X regardless of how an architecture manages direct maps or instruction caches.

Every operation advertises capability and lifecycle before the default policy commits. Missing mandatory capabilities fail strict profile activation; a deliberately compatible profile may boot with a named degraded state. The UAPI reports compact capability codes and never exposes architecture secrets or raw mappings.

Cross-compilers, architecture QEMU machines, configs, and boot assertions are added to `verification/`. Emulation proves build and functional contracts. Release qualification separately records physical boards, firmware, IOMMU behavior, interrupt load, and performance so QEMU cannot be mistaken for hardware evidence.

## Open Questions

None. The accepted defaults keep implementation x86-64 first and make arm64/RISC-V a truthful parity path.

## Out of Scope
- Claiming production support from cross-compilation or QEMU boot alone.
- Requiring identical low-level mechanisms on every architecture.
- Weakening x86-64 to the least capable architecture.
- Initial ports beyond arm64 and RISC-V.
