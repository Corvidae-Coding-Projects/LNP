# Initialization-only code and data reclamation

## Goal
Increase the amount of boot-only code and data that is annotated, verified, and reclaimed after initialization. The design uses call-graph and reference analysis to avoid retaining privileged initialization machinery while preventing unsafe references into freed init sections.

## Requirements
- REQ-1: Inventory functions and data reachable only before `free_initmem()` and classify blockers that keep initialization-only dependency chains resident.
- REQ-2: Apply existing `__init`, `__initdata`, `__initconst`, and related annotations from `include/linux/init.h` where lifetime proof is complete.
- REQ-3: Detect direct, indirect, callback, alternative, and module references from runtime sections into reclaimed init sections at build time.
- REQ-4: Split mixed-lifecycle functions and data so boot-only logic is reclaimed without duplicating security-critical state or changing runtime semantics.
- REQ-5: Poison or unmap reclaimed init memory according to upstream architecture support and produce a controlled fault for deliberate post-init use.
- REQ-6: Preserve boot on supported firmware paths, CPU bring-up, initcalls, late init, module loading, suspend/resume, kexec, and crash-kernel setup.
- REQ-7: Report reclaimed bytes and runtime text/data reduction for each release without turning size reduction into a correctness override.

## How we will know it works
- [ ] AC-1 [REQ-1]: A reproducible call-graph report lists each new candidate, its last legal phase, retained reference reason, and final disposition.
- [ ] AC-2 [REQ-2]: Annotated symbols appear in the expected init ELF sections and the final boot log records their reclamation after the last legal user.
- [ ] AC-3 [REQ-3]: Modpost, objtool, and compiler checks reject seeded direct and function-pointer references from runtime code to an init symbol.
- [ ] AC-4 [REQ-4]: Split functions preserve existing unit and integration results, and no mutable runtime field shares storage with reclaimed init-only data.
- [ ] AC-5 [REQ-5]: A test-only post-init call or write reliably faults and emits a bounded diagnostic rather than executing reclaimed content.
- [ ] AC-6 [REQ-6]: QEMU matrices cover UEFI and BIOS boot, SMP bring-up, module load, suspend/resume where available, kexec, and crash-kernel reservation.
- [ ] AC-7 [REQ-7]: Release verification records init text/data bytes before and after the feature and fails if a regression is unexplained.

## Technical plan
The implementation extends the lifetime annotations in `../hdn-kernel/include/linux/init.h` and the freeing sequence in `../hdn-kernel/init/main.c`. Linker placement remains architecture-owned under `../hdn-kernel/arch/x86/kernel/`; HDN does not introduce a parallel boot lifecycle.

A build-time analyzer consumes compiler call graphs, relocation records, modpost section-mismatch data, and objtool reachability. It distinguishes calls made during boot from function addresses stored for runtime callbacks. A candidate is accepted only when every incoming edge ends before reclamation. Ambiguous assembly or opaque callback registration keeps the symbol resident and appears in the report.

Mixed-lifecycle routines are split at the semantic boundary: parsing and one-time setup become init-only, while the minimum runtime operation remains resident. Security configuration that must be visible later is copied into explicitly sealed runtime storage before init memory is released. CONSTIFY protects static post-init state; KERNSEAL handles dynamic handoff objects.

Verification artifacts are generated through `verification/bin/hdn-verify`, and runtime misuse probes live under `../hdn-kernel/tools/testing/selftests/hardening/`. The production build keeps upstream section mismatch warnings fatal for HDN-touched paths.

## Open questions

No open questions. The current plan requires evidence for each annotation and x86-64 boot testing before expanding the feature.

## Not included
- Annotating code solely to maximize reclaimed byte counts.
- Reclaiming late-init or hotplug code whose lifecycle remains active.
- Changing upstream initcall ordering without a separate design.
- Using post-init faults as a substitute for build-time reachability checks.
