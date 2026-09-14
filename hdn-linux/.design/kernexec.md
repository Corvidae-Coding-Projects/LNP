# Kernel execute and write isolation

## Goal
Define an HDN-native execute-integrity layer that removes writable aliases of kernel executable state and constrains the few transitions that legitimately create or modify executable mappings. The design uses upstream W^X primitives first and adds narrowly scoped x86-64 enforcement where Linux 7.0.12 still exposes mutable GDT, page-table, direct-map, module, or JIT aliases.

## Requirements
- REQ-1: Define and enforce the invariant that a physical page cannot be writable through one kernel alias while executable through another after its construction window closes.
- REQ-2: Replace unrestricted runtime access through `arch/x86/include/asm/desc.h` helpers such as `get_cpu_gdt_rw()` with lifecycle-scoped mutation APIs, while preserving CPU bring-up, suspend, resume, kexec, and virtualization behavior.
- REQ-3: Restrict page-table-page writes to audited update paths in `arch/x86/mm/` and `mm/`, without treating `CONFIG_PAGE_TABLE_CHECK` as equivalent to page-table sealing.
- REQ-4: Cover kernel text, alternatives, static calls, ftrace, kprobes, livepatch, modules, eBPF JIT images, EFI runtime transitions, and crash kernels with explicit construction and finalization states.
- REQ-5: Integrate policy and status through `security/hardening/Kconfig`, `security/hardening/core.c`, and the existing HDN policy ABI without adding an end-user compatibility control.
- REQ-6: Demonstrate that enforcement adds no measurable steady-state cost outside legitimate mapping transitions and fails closed when a transition violates W^X.

## How we will know it works
- [ ] AC-1 [REQ-1]: An LKDTM or HDN selftest proves that attempts to create simultaneous writable and executable aliases are rejected, while `/sys/kernel/debug/kernel_page_tables` shows no unexpected W+X kernel mappings after boot.
- [ ] AC-2 [REQ-2]: GDT mutation tests pass across secondary CPU online/offline, suspend/resume, and a QEMU reboot, and an instrumented unauthorized post-finalization write faults or is rejected.
- [ ] AC-3 [REQ-3]: All page-table write sites found by a repository-wide static checker use the approved mutation API, and negative tests cannot write a sealed page-table page through the direct map.
- [ ] AC-4 [REQ-4]: Kernel selftests for modules, ftrace, kprobes, livepatch where configured, BPF JIT, kexec, and EFI boot pass under the hardened configuration; every temporary writable transition is paired with an audited finalization event.
- [ ] AC-5 [REQ-5]: `hdn-status` reports active, degraded, or unsupported KERNEXEC subfeatures from machine-readable kernel state, and signed policy cannot relax the invariant after runtime sealing.
- [ ] AC-6 [REQ-6]: Kernel build, hackbench, kernel compilation, module load, and BPF JIT benchmarks show no statistically significant steady-state regression; transition microbenchmarks and all denials are recorded in the verification report.

## Technical plan
The authority remains the clean tagged source at `../hdn-kernel`. Configuration is added beside the existing HDN switches in `../hdn-kernel/security/hardening/Kconfig`, while orchestration and status live in `../hdn-kernel/security/hardening/core.c`. The implementation must reuse `../hdn-kernel/arch/x86/mm/pat/set_memory.c`, `../hdn-kernel/mm/execmem.c`, and `../hdn-kernel/kernel/module/strict_rwx.c` instead of creating a second executable-memory subsystem.

The core abstraction is a typed construction token: callers acquire a token for one enumerated transition, mutate the exact range, perform architecture-required cache and TLB synchronization, and irreversibly finalize the range. Tokens are not integers exposed to modules. The implementation records the range, transition reason, owner, and state in an internal table protected by a raw lock suitable for early boot. Unexpected overlap, re-finalization, or writable-plus-executable alias detection terminates the transition and emits a bounded HDN event.

GDT handling is split into boot construction, CPU-hotplug reconstruction, and sealed runtime use. Existing read-only CPU-entry-area mappings remain the execution view; writable backing aliases are made inaccessible except inside the descriptor update primitive. Page-table hardening similarly wraps allocation and mutation rather than relying on `../hdn-kernel/mm/page_table_check.c`, whose purpose is validating mapping relationships rather than protecting the page-table memory itself.

Dynamic text users receive explicit adapters. Alternatives and static calls finish during boot or use stop-machine-protected transitions; modules and BPF allocate non-executable memory, populate it, then finalize it executable and non-writable; ftrace, kprobes, and livepatch use bounded patch windows. Hibernation, kexec, and EFI paths must prove their transition ordering in QEMU before the option can be selected by the release config.

Verification belongs in `../hdn-kernel/tools/testing/selftests/hardening/`, with destructive fault probes isolated to test-only modules. The matrix in `verification/configs/` builds both enforcement-on and dependency-negative configurations, while TCG covers correctness and host KVM covers timing-sensitive SMP, hotplug, and suspend cases.

## Open questions

No open questions. The current plan starts with Linux 7.0.12 on x86-64, adds no user-facing bypass, and requires measurement before wider use.

## Not included
- Replacing upstream x86 page-table APIs wholesale.
- Supporting architectures other than x86-64 in the first implementation.
- Treating confidentiality of kernel virtual addresses as the primary security boundary.
- Enabling arbitrary runtime kernel text modification for compatibility.
