# HDN Linux

HDN Linux is a Linux kernel hardening patchset for a secure-by-default,
daily-driver operating system. Security policy is intended to stay out of the
user's way: normal applications, networking, drivers, and firmware work without
terminal setup, while sensitive administration crosses a graphical PolicyKit
approval boundary.

The current release candidate targets upstream Linux `7.0.12` on x86_64.

> HDN is an independent implementation with its own policy model, interfaces,
> tests, and source structure. Functional comparison does not imply source,
> configuration, or ABI compatibility with grsecurity or PaX.

## At A Glance

| Item | Current result |
| --- | --- |
| Kernel base | Linux `7.0.12` |
| Product target | x86_64 daily-driver desktop |
| Equivalence matrix | 68/68 rows dispositioned |
| Covered or upstream baseline | 66/68 rows (97.1%) |
| Permanently deferred | 2/68 rows |
| QEMU hardening smoke | 1,111/1,111 pass |
| Kallsyms API isolation | 3/3 pass |
| Generic package build | 4,970 signed modules, 0 unsigned |
| Full OS validation | Release-36 live, graphical install, reboot, adversarial, and application QA passed |

The two permanent deferrals are obsolete executable-memory emulation and exact
PaX/grsecurity compiler-plugin or non-x86 architecture implementations beyond
the selected upstream equivalents. See the
[functional equivalence matrix](docs/HDN_GRSEC_EQUIVALENCE.md) for the evidence
and disposition of every row.

## Security Model

### Policy And Authority

- Kernel-trusted, signed policy blobs with transactional commit, rollback, and
  one-way runtime sealing.
- Executable, account, and inheritance-based profiles with least-authority
  gates for sensitive kernel operations. Sandbox and container profiles can
  retain those bounds across executables visible only in a runtime mount
  namespace, without weakening privileged-exec transitions.
- Per-profile capability ceilings, resource ceilings, umask floors, and
  cross-profile process controls.
- Typed, signed object rules for files, directory trees, mounts, descriptors,
  devices, ptrace, and filesystem or abstract AF_UNIX sockets.

### Memory And Exploit Mitigation

- Global W+X prohibition, executable-stack and text-relocation controls, RELRO
  sealing, and narrowly scoped JIT compatibility policy.
- Hardened upstream compiler, allocator, initialization, stack, page-table,
  kernel W^X, module W^X, and address-randomization baselines.
- Locked security floors for low-address mappings, userfaultfd, BPF JIT,
  protected paths, coredumps, Yama, TIOCSTI, and related sysctls.

### Kernel Attack Surface

- Forced module signatures plus signed-policy admission for explicit and
  automatic module loading.
- Profile gates for BPF, perf, io_uring, firmware, debugfs, kernel logs, raw
  devices, USB monitoring, kexec, user namespaces, and other high-risk APIs.
- Kernel-symbol and address redaction across procfs, sysfs, tracing, BPF, BTF,
  kallsyms, and formatted-symbol interfaces.

### Process And Data Isolation

- Cross-process procfs disclosure controls for memory, argv, environment,
  credentials, scheduling, I/O, limits, and live syscall metadata.
- Object-operation enforcement across read, write, append, exec, create,
  delete, rename, link, metadata, xattr, mount, ioctl, locking, fd passing,
  and directory discovery paths.
- Hardened chroot, signal, terminal, IPC, socket, privileged-exec, and
  cross-profile ptrace behavior.

### Audit And Operations

- Stable structured kernel events with decoding, flood suppression, policy
  learning, and persistent journald collection.
- Product-facing status, compatibility status, and bounded support bundles
  without exposing raw securityfs details.
- Read-only system image sealing with fixed-operation package update and repair
  transactions.

### Desktop Administration

- PolicyKit-backed administrator approval for typed actions rather than raw
  shell commands.
- One authorization per action, with preflight and execution kept inside the
  same authorized root session.
- Native security-center, update, repair, rollback, recovery, and policy
  workflow backends.
- Fail-closed root-owned manifests, one-use approval tokens, fixed-argument
  workers, preflight validation for privileged operations, and a token-exempt
  monotonic boot intent that can only make approved mounts read-only.
- Package-service quiescing and bounded busy-writer retries keep the protected
  image resealed after graphical update transactions.

The detailed hook map and rationale live in the
[kernel hardening design](docs/KERNEL_HARDENING_DESIGN.md). The complete
adversarial feature accounting and QEMU evidence live in the
[equivalence matrix](docs/HDN_GRSEC_EQUIVALENCE.md).

## Repository Layout

| Path | Purpose |
| --- | --- |
| `patches/hdn-linux-7.0.12.patch` | Generated patch against clean Linux 7.0.12 |
| `docs/KERNEL_HARDENING_DESIGN.md` | Architecture, policy model, and hook map |
| `docs/HDN_GRSEC_EQUIVALENCE.md` | Functional comparison, evidence, and deferrals |
| `docs/QA_REPORT.md` | Exact-ISO release-36 QA matrix, observations, and limits |
| `docs/SOURCE_CONTROL.md` | Kernel source, patch, ABI, package, and OS release identity |
| `scripts/verify-release-patch.sh` | Deterministic source and patch verification |
| `LICENSE` | GPL-2.0 license |

## Apply The Patch

```sh
tar -xf linux-7.0.12.tar.xz
cd linux-7.0.12
patch -p1 < ../hdn-linux/patches/hdn-linux-7.0.12.patch
```

## Build

Use an out-of-tree build directory:

```sh
make O=/path/to/build olddefconfig
make O=/path/to/build -j"$(nproc)" bzImage modules
```

The product configuration enables the `SECURITY_HARDENING` feature set and its
release-scoped mitigation baseline. A generic distro configuration should be
reviewed before deployment instead of assuming that `olddefconfig` selects a
complete product policy.

## Verification

Current release-candidate evidence:

```text
QEMU hardening smoke:          1111/1111 pass
Kallsyms API isolation:        3/3 pass
Generic x86_64 package build:  4,970 signed modules, 0 unsigned
```

Full-system release evidence:

| Check | Result |
| --- | --- |
| ISO | `HDN-OS-Live.x86_64-1.0.0.iso` |
| ISO SHA-256 | `a13c59b1ed27329f7b95d09732695143df2c247645b6a6a8e6e590b3b7e7bad0` |
| ISO package baseline | `kernel-hdn 7.0.12-30`, `hdn-tools 7.0.12-23`, `hdn-release 1-16`, `hdn-desktop 1-8`, `hdn-branding 1-5` |
| Boot path | UEFI Secure Boot with the HDN development certificate enrolled in OVMF |
| Installed system | Btrfs root writable; mount-local `/usr` self-bind read-only |
| Security state | HDN policy loaded and sealed; SELinux enforcing; Secure Boot enabled |
| Kernel red team | 20/20 expected allow and deny results passed as an unprivileged desktop user |
| Application QA | 24/24 bundled apps; Flatpak; rootless Podman; GIMP, VLC, Audacity, Inkscape, and Thunderbird |
| Health after stress and reboot | HDN services active; zero failed units, AVCs, coredumps, or kernel-critical events |

The exact ISO passed its embedded media check, a read-only nested-filesystem
check, and byte-for-byte verification of 118,755 staged `/usr` files. It then
booted live under Secure Boot, passed a 100-session OpenSSH PTY stress run, and
completed the desktop, sandbox, container, module, firmware-service, and image
seal checks. The same image completed a graphical Anaconda installation to a
fresh virtio disk. The installed system booted twice under Secure Boot and
retained its policy state and common application installs across reboot.

The PTY regression found in kernel release 28 is closed in release 30. Exact
allocator, descriptor-receiver, and session-parent provenance preserves the
OpenSSH privilege-separation handoff without granting ordinary inherited PTY
access. Flatpak and container compatibility fixes remain restricted to their
signed profiles. See the [release QA report](docs/QA_REPORT.md) for the full
matrix and remaining qualification limits. Production Secure Boot still needs
a generally trusted signing path or explicit certificate enrollment; the QA
environment uses the HDN development certificate enrolled in OVMF.

Patch artifact:

```text
path:    patches/hdn-linux-7.0.12.patch
lines:   79,805
bytes:   2,415,724
sha256:  4b3ee5258aa72cca24ad1f11087666e745d131b9d05fa765b8adb9f06b659cf4
```

Validate it against a clean Linux 7.0.12 tree with either tool:

```sh
patch --dry-run -p1 -d /path/to/linux-7.0.12 \
  < patches/hdn-linux-7.0.12.patch

git -C /path/to/linux-7.0.12 apply --check --whitespace=error \
  /path/to/hdn-linux/patches/hdn-linux-7.0.12.patch
```

## Scope

HDN tracks three different completion levels:

| Level | Status |
| --- | --- |
| Daily-driver security and usability | Release-candidate integration complete in QEMU; physical hardware qualification remains |
| Broad grsecurity-style security effects | 66 covered/baseline rows; all release-scoped rows closed |
| Exact PaX/compiler/architecture parity | Permanently out of scope where documented |

HDN does not clone grsecurity's RBAC language, configuration names, sysctl
names, group selectors, text-log strings, private update process, or
architecture-specific implementations. Expansion to additional operations,
architectures, and compiler mechanisms is future work, not an untracked claim
of current parity.

## Development Workflow

1. Make functional changes as commits on the HDN kernel source branch.
2. Run focused builds and tests for the changed enforcement surface.
3. Run the full QEMU hardening smoke suite when behavior changes.
4. Regenerate the patch from the named source tag and clean upstream archive.
5. Run `scripts/verify-release-patch.sh`.
6. Update the artifact metadata above and publish a checkpoint commit.

See [kernel source and release identity](docs/SOURCE_CONTROL.md) for the exact
release-30 commit chain and the relationship among source, patch, package,
policy ABI, and OS release.

## License

HDN Linux kernel changes are licensed under GPL-2.0, matching the Linux kernel
patch licensing requirement. See [LICENSE](LICENSE).
