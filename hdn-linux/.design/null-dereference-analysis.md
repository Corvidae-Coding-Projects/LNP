# Optimized null-dereference build analysis

## Goal
Add a compiler-assisted reporting gate for potential null dereferences that remain visible after optimization, then route findings through review and targeted fixes. This is a correctness and exploitability-reduction tool, not runtime null-page emulation or a license to suppress compiler diagnostics broadly.

## Requirements
- REQ-1: Report dereferences, offset accesses, calls, and container conversions whose optimized dataflow still admits a null base on a reachable path.
- REQ-2: Distinguish definite faults, attacker-influenced potential faults, invariant-dependent paths, compiler artifacts, and intentional probes with stable categories.
- REQ-3: Correlate findings with existing Smatch, Sparse, compiler warnings, and Coccinelle results to reduce duplicate review without hiding disagreement.
- REQ-4: Require source-level repair for definite and attacker-reachable findings; allow suppressions only for locally provable invariants or architecture-defined probes.
- REQ-5: Keep null-page mappings prohibited by the existing mmap minimum and ensure runtime faults retain ordinary oops, recovery, and HDN event behavior.
- REQ-6: Run analysis across the release configuration, sanitizer configurations, and representative optional subsystems because reachability and optimization vary by config.
- REQ-7: Maintain a zero-unreviewed-finding gate for newly changed code and a burn-down ledger for accepted preexisting findings.

## How we will know it works
- [ ] AC-1 [REQ-1]: Compiler fixtures detect seeded direct, field-offset, indirect-call, and `container_of` null paths after optimization and emit source-linked stable identifiers.
- [ ] AC-2 [REQ-2]: Every report carries one category, reachability evidence, taint status, and optimization context; reviewers can reproduce it with the recorded command.
- [ ] AC-3 [REQ-3]: The merged report preserves tool provenance and highlights conflicting results rather than deduplicating them away.
- [ ] AC-4 [REQ-4]: CI rejects seeded definite and attacker-reachable findings and rejects a suppression without a local invariant, owner, source span, and invalidation condition.
- [ ] AC-5 [REQ-5]: Tests prove low-address mappings remain denied and a forced kernel null dereference follows the configured controlled failure path without executing user memory.
- [ ] AC-6 [REQ-6]: Baseline, analysis, KASAN, KCSAN, KMSAN, and fuzz configuration reports are generated and tied to their exact `.config` and compiler version.
- [ ] AC-7 [REQ-7]: Diff-aware CI admits no new unreviewed identifier, and the ledger fails when an entry disappears, changes source meaning, or loses its justification.

## Technical plan
The analyzer integrates with the release GCC pipeline through `../hdn-kernel/scripts/gcc-plugins/` and emits structured records consumed by `verification/bin/hdn-verify`. Existing Sparse, Smatch, and Coccinelle stages remain independent producers so correlated output cannot manufacture consensus.

Analysis occurs after enough optimization to expose compiler-retained null paths but before source mapping is lost. It tracks nullability through PHI nodes, field offsets, inlining, container conversions, and indirect calls. Findings are keyed by semantic function and source span plus analyzer version, not generated object addresses.

The triage ledger is versioned beside verification configuration. Definite or attacker-controlled paths fail immediately. Invariant-dependent findings require a source assertion or structural rewrite that other tools can understand. Intentional architecture probes use narrow annotations defined with the owning subsystem rather than a global no-check attribute.

Runtime behavior remains governed by low-address protections and normal fault handling in the kernel; the feature does not map page zero. Regression probes live under `../hdn-kernel/tools/testing/selftests/hardening/`, while analysis reports and compiler identities are retained as external verification artifacts.

## Open questions

No open questions. The current plan starts with GCC, fixes source problems, and allows only closely reviewed suppressions.

## Not included
- Mapping page zero to keep buggy kernel code running.
- Automatically rewriting every analyzer finding.
- Treating absence of reports as proof of memory safety.
- Suppressing all findings in an entire file or subsystem.
