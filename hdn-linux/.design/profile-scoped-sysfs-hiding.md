# Feature: Profile-Scoped Sysfs Discovery Reduction

## Summary
Reduce unnecessary hardware and kernel-topology disclosure through sysfs for constrained applications while preserving the device discovery required by the desktop, udev, system services, and accessibility software. Visibility is derived from signed HDN profiles and namespace context, not a brittle system-wide hide switch.

## Requirements
- REQ-1: Inventory sensitive sysfs classes and attributes by disclosure value, mutability, current permission, and legitimate consumer.
- REQ-2: Enforce visibility at lookup, readdir, link traversal, and open so hidden entries cannot be recovered through alternate sysfs paths or preopened descriptors.
- REQ-3: Bind restrictions to authenticated HDN application profiles, mount/user namespace context, and service identity while leaving the trusted base system functional.
- REQ-4: Preserve udev, systemd, NetworkManager, power management, graphics, audio, storage, firmware update, containers, Flatpak, accessibility, and hardware hotplug behavior.
- REQ-5: Treat hiding as attack-surface and discovery reduction rather than a confidentiality boundary against a process with stronger kernel observation capabilities.
- REQ-6: Provide stable brokered answers for approved application needs instead of granting broad raw sysfs visibility.
- REQ-7: Test information reduction, bypass resistance, desktop compatibility, hotplug, suspend/resume, and performance of sysfs-heavy workloads.

## Acceptance Criteria
- [ ] AC-1 [REQ-1]: The inventory identifies each selected node family, exposed information, writers/readers, profile decision, broker alternative, and residual side channel.
- [ ] AC-2 [REQ-2]: Negative tests cannot discover a hidden node by directory enumeration, direct path lookup, symlink, bind mount, alternate class/device path, or descriptor transfer across a restricted boundary.
- [ ] AC-3 [REQ-3]: A signed restricted profile sees its declared view; unprofiled trusted services retain the base view; executable replacement and namespace transition invalidate stale decisions.
- [ ] AC-4 [REQ-4]: Automated live-system tests pass for device enumeration, networking, graphics, audio, storage, fwupd, containers, Flatpak, accessibility, hotplug, and suspend/resume.
- [ ] AC-5 [REQ-5]: Documentation and `hdn-status` describe the feature as discovery reduction, and tests record equivalent information still obtainable through intentionally public APIs.
- [ ] AC-6 [REQ-6]: Broker tests return only the declared stable fields, authenticate the caller, redact unneeded topology, and remain correct through device add/remove.
- [ ] AC-7 [REQ-7]: Repeated sysfs traversal and desktop startup benchmarks stay within the accepted profile budget, and the bypass corpus has zero unexplained disclosures.

## Architecture
Policy decisions originate in `../hdn-kernel/security/hardening/core.c` and use the existing executable-identity and sealed-profile model. Enforcement hooks are placed at the kernfs/sysfs boundary under `../hdn-kernel/fs/kernfs/` and `../hdn-kernel/fs/sysfs/` so enumeration and direct lookup share one decision rather than diverging per subsystem.

Each profile references a versioned visibility class, not arbitrary pathname globbing. Kernel registration associates kernfs node types with stable disclosure categories. At lookup and enumeration, the hook evaluates the current authenticated subject, namespace, node category, and operation. Decisions are cached only with policy generation and namespace identity, preventing stale visibility after profile replacement.

Base-system daemons retain their necessary view under narrow service identities. Sandboxed applications receive brokered hardware facts through distro services when a stable high-level answer suffices. The broker belongs in `../hdn-os/packages/hdn-tools/` and never proxies arbitrary sysfs paths.

Tests combine kernel fixtures under `../hdn-kernel/tools/testing/selftests/hardening/` with the live application and service suites in `../hdn-os/tests/`. Full-system validation includes hotplug and suspend because device recreation can otherwise bypass cached categorization.

## Open Questions

None. The accepted defaults select profile-scoped visibility and preserve the trusted desktop/service base.

## Out of Scope
- Claiming sysfs hiding protects against root or arbitrary kernel read primitives.
- System-wide removal of sysfs required by the operating system.
- Path-glob policy supplied directly by applications.
- Concealing information intentionally exported through unrelated public APIs.
