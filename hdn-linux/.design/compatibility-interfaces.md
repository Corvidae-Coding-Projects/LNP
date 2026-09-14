# Feature: HDN Compatibility and Policy Translation Interfaces

## Summary
Provide stable HDN-native adapters for software that historically expects PaX flags, grsecurity administration concepts, security sysctls, group controls, or recognizable denial categories. The layer translates intent into signed HDN profiles without impersonating unsupported legacy semantics or exposing a global weakening switch.

## Requirements
- REQ-1: Inventory real applications, packaging tools, and operational workflows that inspect or set PaX ELF flags, invoke `gradm`, write legacy sysctls, depend on special groups, or parse grsecurity-style logs.
- REQ-2: Define a versioned HDN compatibility-intent schema that maps supported legacy requests to existing signed policy capabilities and rejects non-equivalent requests explicitly.
- REQ-3: Authenticate executable and package identity before applying compatibility intent, and invalidate grants when executable content or policy generation changes.
- REQ-4: Keep translation in userspace wherever kernel behavior already exists, adding kernel UAPI only for a missing enforceable primitive.
- REQ-5: Present generic actionable status through HDN tools and desktop integration without requiring ordinary users to understand PaX or grsecurity terminology.
- REQ-6: Preserve sealed policy monotonicity: compatibility translation may narrow behavior or select preapproved capability sets but cannot relax runtime-sealed global invariants.
- REQ-7: Test package installation, upgrades, rollback, containers, Flatpak boundaries, malformed metadata, races, and unsupported legacy requests end to end.

## Acceptance Criteria
- [ ] AC-1 [REQ-1]: A machine-readable inventory names each consumer, observed legacy interface, required behavior, HDN disposition, and regression fixture; no interface is implemented solely because it existed historically.
- [ ] AC-2 [REQ-2]: Schema conformance tests accept every supported intent, reject unknown or semantically broader requests, and retain backward compatibility across documented schema versions.
- [ ] AC-3 [REQ-3]: Renamed, replaced, mutated, unsigned, or rolled-back executables cannot inherit a stale grant; a verified package update can obtain only its newly compiled profile.
- [ ] AC-4 [REQ-4]: Kernel ABI review shows every added operation enforces a primitive unavailable through the current interface in `include/uapi/linux/hardening.h`; all other translation remains in `tools/hardening/`.
- [ ] AC-5 [REQ-5]: `hdn-status`, support bundles, and the HDN security center display application-level remediation and stable event categories without raw policy internals.
- [ ] AC-6 [REQ-6]: Attempts to translate an intent that would disable KERNEXEC, UDEREF, RAP, module signing, or another sealed mandatory invariant fail before policy mutation.
- [ ] AC-7 [REQ-7]: Distro integration tests cover install, update, rollback, containers, malformed metadata, concurrent launch/update, and every unsupported-intent error path.

## Architecture
The existing kernel interface is `../hdn-kernel/include/uapi/linux/hardening.h`, with policy orchestration in `../hdn-kernel/security/hardening/core.c`. Userspace policy compilation and status already live in `../hdn-kernel/tools/hardening/`; those tools remain the translation authority and are packaged by the sibling `../hdn-os/packages/hdn-tools/` integration.

An offline compiler consumes package-owned compatibility declarations, verifies their provenance, and emits ordinary HDN policy fragments. Legacy ELF flags are inputs to migration tooling, not mutable runtime authority. A `gradm`-named shim is provided only if a real consumer cannot be migrated; it reports unsupported commands precisely and never claims to configure an RBAC system that HDN does not implement.

Legacy sysctl and group concepts map to explicit policy capabilities tied to authenticated executables or service identities. No magic group membership grants ambient privilege. Logging translation occurs when decoding stable HDN event codes, allowing support tools to label familiar categories without changing kernel log strings or exposing sensitive fields.

The HDN OS tests under `../hdn-os/tests/` exercise package transactions and desktop presentation. Kernel selftests verify policy monotonicity, identity invalidation, and unsupported operations. The release manifest records schema and policy-compiler versions so rollback cannot combine incompatible components silently.

## Open Questions

None. The accepted defaults select HDN-native semantics, userspace-first translation, and signed per-application profiles.

## Out of Scope
- Reimplementing the full grsecurity RBAC system or `gradm` command surface.
- Advertising binary compatibility where semantics differ.
- Ambient privilege based only on numeric group membership.
- Global runtime switches that disable sealed hardening.
