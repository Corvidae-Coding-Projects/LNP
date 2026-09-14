# Private kernel stack mappings

## Goal
Remove ordinary kernel-stack pages from broadly reusable virtual aliases and expose only the current task's guarded stack mapping to normal execution. This narrows stack-disclosure and stack-pivot primitives beyond upstream `VMAP_STACK` while retaining debuggability through controlled privileged paths.

## Requirements
- REQ-1: Allocate kernel stack backing pages outside ordinary slab reuse and map them only through guarded private-stack virtual regions during task execution.
- REQ-2: Prevent a writable direct-map or stale task mapping from remaining accessible after the stack becomes private or the task switches out.
- REQ-3: Switch private mappings safely across context switch, interrupt, NMI, exception, CPU hotplug, idle, and task exit without exposing another task's active stack.
- REQ-4: Preserve ORC unwinding, scheduler diagnostics, ptrace-visible semantics, crash dumps, lockdep, KASAN where supported, and stack-depot consumers through privileged translation helpers.
- REQ-5: Zero and invalidate stack storage before reuse, use guard pages on both ends, and detect stack overflow without relying on adjacent mapped memory.
- REQ-6: Coordinate with RAP return integrity so protected return state cannot be bypassed through an alternate writable stack alias.
- REQ-7: Measure context-switch, interrupt, syscall, fork/exit, memory, and TLB overhead before making private stacks default.

## How we will know it works
- [ ] AC-1 [REQ-1]: Page-table inspection shows active stacks only in the designated guarded region and allocator tests show backing pages never enter a general-purpose slab cache.
- [ ] AC-2 [REQ-2]: Negative tests cannot access a private stack through the direct map or a stale task virtual address after switch-out or free.
- [ ] AC-3 [REQ-3]: SMP stress covers rapid task switching, nested interrupts, NMIs, exceptions, CPU offline/online, idle transitions, and task teardown without stale or missing mappings.
- [ ] AC-4 [REQ-4]: ORC unwind, SysRq task dumps, lockdep, crash-kernel collection, KASAN-supported configurations, and stack traces retain correct task attribution without globally remapping all stacks.
- [ ] AC-5 [REQ-5]: Overflow tests hit an unmapped guard, and reuse tests prove the entire accessible stack is zeroed before assignment to a new task.
- [ ] AC-6 [REQ-6]: RAP corruption tests find no alternate stack alias and preserve return authentication across task and interrupt stack transitions.
- [ ] AC-7 [REQ-7]: Repeated microbenchmarks and workloads report context switches, syscalls, interrupts, fork/exit, page-table memory, and TLB costs with confidence intervals.

## Technical plan
The implementation extends the thread-stack lifecycle in `../hdn-kernel/kernel/fork.c`, x86 switching in `../hdn-kernel/arch/x86/kernel/process_64.c`, and stack definitions in `../hdn-kernel/arch/x86/include/asm/thread_info.h`. It builds on upstream `VMAP_STACK` and architecture page-table helpers rather than maintaining an unrelated allocator.

Backing pages are allocated as a guarded stack object. Before publication, KERNEXEC removes writable direct-map access for those physical pages and installs the mapping in a per-CPU private-stack virtual window. Context switch changes the window to the incoming task with architecture-required TLB handling. Interrupt and exception stacks remain separately mapped, domain-labeled regions and cannot alias task stacks.

Inactive tasks retain backing metadata but not a generally dereferenceable kernel virtual address. Debug and crash paths use a privileged translation helper that validates task lifetime and temporarily maps only the requested read range; normal code cannot call this helper. Task exit unmaps, waits for remote observers, scrubs backing pages, and returns them to the dedicated allocator.

RAP stores return-integrity context outside attacker-writable stack backing or authenticates it against the private mapping generation. Status and bounded failures use `../hdn-kernel/security/hardening/core.c`. Verification combines page-table probes, context-switch stress, unwinder tests, fault injection, and host-KVM performance runs.

## Open questions

No open questions. The current plan starts with x86-64, uses guarded mapping windows for each CPU, and requires measurement before enabling it by default.

## Not included
- Encrypting kernel stack contents in RAM.
- Making all task stacks visible to ordinary procfs or debugfs users.
- Initial support for non-x86 architectures.
- Disabling crash diagnostics rather than adapting them safely.
