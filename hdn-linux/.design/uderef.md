# Feature: UDEREF-Style User and Kernel Address Separation

## Summary
Strengthen the boundary between user pointers and kernel accesses so that user memory is reachable only through explicit, auditable uaccess windows. Hardware SMAP remains the x86-64 fast path, with static verification and an optional software fallback for supported CPUs that lack SMAP.

## Requirements
- REQ-1: Keep SMAP enabled and pinned on capable x86-64 systems, and expose a truthful degraded state when hardware support is absent.
- REQ-2: Require all intentional user-memory access to flow through the primitives in `arch/x86/include/asm/uaccess.h` and reject unbalanced or nested access-window misuse.
- REQ-3: Add build-time analysis that finds direct user-pointer dereferences, unsafe address-space casts, and uaccess windows that cross sleeping or callback boundaries.
- REQ-4: Provide a software enforcement mode for explicitly supported non-SMAP x86-64 systems without weakening the hardware-backed mode or silently claiming equivalence.
- REQ-5: Preserve ptrace, signal delivery, io_uring, userfaultfd, virtualization, compat tasks, and high-throughput read/write behavior.
- REQ-6: Integrate denials and degradation with the bounded HDN event channel and sealed policy, without exposing raw address data to ordinary users.

## Acceptance Criteria
- [ ] AC-1 [REQ-1]: Boot tests on SMAP-capable QEMU and hardware show CR4.SMAP set after policy sealing, and a deliberate attempt to clear it is restored or rejected and logged.
- [ ] AC-2 [REQ-2]: Instrumented selftests prove ordinary kernel dereferences of user mappings fault while `copy_from_user()`, `copy_to_user()`, and approved nofault helpers continue to work.
- [ ] AC-3 [REQ-3]: Objtool and semantic-patch checks run in CI with zero unexplained direct user-pointer dereferences or access windows spanning a scheduler, callback, or exception-unsafe boundary.
- [ ] AC-4 [REQ-4]: A no-SMAP TCG CPU profile either boots with the documented software mode and passes the isolation tests or reports unsupported; it never reports hardware-equivalent enforcement.
- [ ] AC-5 [REQ-5]: Relevant kernel selftests and HDN regression tests for ptrace, signals, io_uring, userfaultfd, KVM guest I/O, 32-bit compatibility where enabled, and bulk I/O pass.
- [ ] AC-6 [REQ-6]: `hdn-status` distinguishes hardware, software, and unsupported states; audit events identify the call site by stable kernel build identity rather than leaking a raw pointer.

## Architecture
The hardware path builds on CR4 pinning in `../hdn-kernel/arch/x86/kernel/cpu/common.c` and the existing access-window assembly in `../hdn-kernel/arch/x86/include/asm/uaccess.h`. The HDN switch and dependency rules reside in `../hdn-kernel/security/hardening/Kconfig`; state reporting uses `../hdn-kernel/security/hardening/core.c` and the existing policy/status interface.

Access windows become lexically paired operations with debug-state tracking available in analysis builds. Entry code must close user access before invoking callbacks, scheduling, or returning through an exception path. Objtool gains rules that understand STAC/CLAC and equivalent macros, while Coccinelle checks reject plain casts that discard `__user`. The production fast path remains the same small instruction sequence when SMAP is available.

The software fallback is separately named and reported. It may use constrained page-table or address-limit techniques only if they are compatible with current x86-64 entry code; it cannot reintroduce the historical `set_fs()` model. If the measured cost or architectural complexity is unacceptable, supported no-SMAP machines remain in an honestly degraded profile rather than weakening capable systems.

Tests live in `../hdn-kernel/tools/testing/selftests/hardening/` and include exception fixups, nested interrupts, preemption, NMI observation, and malicious user mappings. Static checks run through `verification/bin/hdn-verify`; TCG supplies selectable CPU feature sets and host KVM validates realistic interrupt and I/O behavior.

## Open Questions

None. The accepted defaults require hardware-first enforcement, truthful fallback reporting, and x86-64 delivery before cross-architecture work.

## Out of Scope
- Reintroducing `set_fs()` or a process-wide address-limit override.
- Hiding whether a CPU lacks SMAP.
- Replacing normal Linux uaccess APIs with an HDN-only userspace ABI.
- Initial arm64, RISC-V, or 32-bit architecture implementation.
