# Feature: Static Object Constification and Post-Init Immutability

## Summary
Expand immutable-after-initialization coverage for static kernel objects and detect writes that escape their declared initialization lifecycle. The feature combines source annotations, linker placement, write-site analysis, and runtime page permissions while preserving legitimate update mechanisms such as static keys and alternatives.

## Requirements
- REQ-1: Inventory writable static objects that are initialized once and thereafter consumed as configuration, dispatch tables, credentials, policy, or security metadata.
- REQ-2: Convert eligible objects to `const` or `__ro_after_init` using the definitions in `include/linux/cache.h` and existing linker sections.
- REQ-3: Prove that every post-init write to a candidate is removed, moved into an explicitly mutable subobject, or mediated by a separately designed update primitive.
- REQ-4: Exclude legitimate runtime-mutated facilities such as static keys, alternatives, tracepoints, jump labels, counters, and hotplug state unless their lifecycle supports sealing.
- REQ-5: Add build-time checks for function-pointer tables and security-sensitive static objects that remain writable without a machine-readable justification.
- REQ-6: Verify that protected sections are read-only after initialization and remain protected across module load, suspend/resume, kexec preparation, and CPU hotplug.

## Acceptance Criteria
- [ ] AC-1 [REQ-1]: A generated inventory records each candidate symbol, type, defining path, initialization writes, runtime reads, address-taking behavior, and disposition.
- [ ] AC-2 [REQ-2]: Converted objects land in read-only ELF or `__ro_after_init` sections as appropriate, confirmed by `readelf`, `nm`, and runtime page-table inspection.
- [ ] AC-3 [REQ-3]: Coccinelle and compiler-assisted write analysis report no unexplained write to a converted object after its seal phase.
- [ ] AC-4 [REQ-4]: Static-key, alternatives, tracepoint, hotplug, and counter regression suites pass, and excluded objects carry a precise semantic classification rather than a blanket file exemption.
- [ ] AC-5 [REQ-5]: CI rejects a seeded mutable security dispatch table and accepts only an exception naming the symbol, owner, required mutation lifecycle, and review condition.
- [ ] AC-6 [REQ-6]: Fault tests cannot write protected storage after init; suspend/resume, module, kexec, and CPU-hotplug tests complete without temporary global unsealing.

## Architecture
Source annotations use existing facilities in `../hdn-kernel/include/linux/cache.h` and section layout in the architecture linker scripts under `../hdn-kernel/arch/x86/kernel/`. The HDN configuration and status bit live in `../hdn-kernel/security/hardening/Kconfig` and `../hdn-kernel/security/hardening/core.c`, but ordinary const-qualified objects require no runtime registration.

The analysis pipeline first identifies static storage whose writes dominate boot initialization and whose later uses are reads or indirect calls. It then classifies candidates as compile-time `const`, boot-populated `__ro_after_init`, split-object, legitimately mutable, or dynamically sealable. Function-pointer aggregates receive priority because a write primitive against them becomes a control-flow primitive.

Objects that mix immutable policy with mutable counters are structurally split so page protection does not force unsafe write windows. Static keys, alternatives, and jump labels retain their upstream mechanisms; CONSTIFY does not wrap them in broad `set_memory_rw()` calls. KERNEXEC enforces the final page permissions, while KERNSEAL handles dynamically allocated objects with later seal points.

Verification uses source analysis through `verification/bin/hdn-verify`, ELF section checks on `vmlinux`, and runtime probes under `../hdn-kernel/tools/testing/selftests/hardening/`. Reports include coverage deltas so new mutable dispatch tables cannot enter unnoticed.

## Open Questions

None. The accepted defaults select upstream annotations first and prohibit compatibility-driven global unsealing.

## Out of Scope
- Dynamically allocated objects, which belong to KERNSEAL.
- Making legitimate counters or hotplug state immutable.
- Runtime mutation by arbitrary modules.
- Treating linker placement alone as proof that no writable alias exists.
