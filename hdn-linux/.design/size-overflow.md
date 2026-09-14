# Allocation and object-size overflow hardening

## Goal
Prevent attacker-controlled integer overflow from producing undersized allocations, copies, or object layouts. The design combines upstream checked-size APIs, structural annotations, static analysis, and narrowly scoped compiler instrumentation instead of enabling the currently experimental global integer-wrap sanitizer as a production shortcut.

## Requirements
- REQ-1: Inventory allocation, reallocation, flexible-array, copy, and length-calculation sites reachable from untrusted inputs, prioritizing multiplication and add-then-multiply expressions.
- REQ-2: Convert eligible calculations to `size_mul()`, `size_add()`, `struct_size()`, `array_size()`, `check_mul_overflow()`, and related upstream helpers.
- REQ-3: Add `__counted_by` and flexible-array metadata where object layout permits the compiler and sanitizers to validate runtime bounds.
- REQ-4: Provide build-time instrumentation or analysis for residual size-producing arithmetic while allowing explicit, reviewed wrapping operations with typed annotations.
- REQ-5: Define failure behavior per call class: return `-EOVERFLOW` or `-E2BIG` at syscall boundaries, fail allocation internally, and never continue with a truncated size.
- REQ-6: Prove compatibility with networking, filesystems, io_uring, BPF, IPC, compat syscalls, and 32-bit-size inputs on the x86-64 kernel.
- REQ-7: Keep runtime cost within the measured default-profile budget and prevent the analysis exception set from growing without review.

## How we will know it works
- [ ] AC-1 [REQ-1]: A reproducible analyzer emits a source-linked inventory of untrusted size expressions with every high-risk finding converted, rejected, or explicitly justified.
- [ ] AC-2 [REQ-2]: Seeded overflow cases at converted sites return a safe error or allocation failure and never reach the allocator or copy primitive with a wrapped value.
- [ ] AC-3 [REQ-3]: Compiler bounds diagnostics and KASAN tests recognize the annotated flexible-array extent for representative network, filesystem, and IPC structures.
- [ ] AC-4 [REQ-4]: The build gate catches seeded unchecked multiplication and addition, accepts annotated intentional wrap, and records every exception in a machine-readable allowlist.
- [ ] AC-5 [REQ-5]: Syscall and internal fault-injection tests verify exact error behavior and confirm no partial object publication occurs after overflow detection.
- [ ] AC-6 [REQ-6]: Relevant kselftests, LTP-style syscall cases, syzkaller programs, and 32-bit compat tests pass with the hardening enabled.
- [ ] AC-7 [REQ-7]: Allocation-heavy, networking, storage, and kernel-build benchmarks report the overhead of checks; CI rejects undocumented allowlist growth.

## Technical plan
Checked arithmetic uses the APIs in `../hdn-kernel/include/linux/overflow.h` and allocation helpers exposed through `../hdn-kernel/include/linux/slab.h`. Flexible-array conversions follow compiler annotations already used throughout `../hdn-kernel/include/` rather than defining an HDN-only layout attribute.

The first phase is source hardening driven by Coccinelle and Smatch through `verification/bin/hdn-verify`. Expressions are classified by destination: allocation size, copy length, array index, protocol length, or intentional modular arithmetic. This avoids the false assumption that all unsigned wrap is invalid. The exception file keys entries by stable semantic identifier and fails closed when the referenced expression disappears or changes.

Residual instrumentation is implemented only for size-producing dataflow that static conversion cannot express cleanly. It is configured in `../hdn-kernel/security/hardening/Kconfig`, carries no user ABI, and must coexist with KASAN, KMSAN, UBSAN bounds checks, and compiler optimization. `../hdn-kernel/lib/Kconfig.ubsan` remains evidence that global integer-wrap sanitization is not itself a production-complete design.

Tests under `../hdn-kernel/tools/testing/selftests/hardening/` feed boundary values through real syscall and parser paths, inspect exact errors, and use allocator fault probes to prove that wrapped sizes are never consumed. Fuzzing retains overflow checks in the fuzz profile even if production instrumentation is narrower.

## Open questions

No open questions. The current plan uses checked source APIs first, adds targeted checks where needed, and requires benchmarks before default use.

## Not included
- Defining all unsigned arithmetic wrap as erroneous.
- Enabling experimental global UBSAN integer-wrap handling as the sole mitigation.
- Changing stable userspace structure layouts without an ABI migration.
- Silently clamping attacker-controlled sizes and continuing execution.
