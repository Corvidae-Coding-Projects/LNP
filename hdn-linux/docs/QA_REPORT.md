# HDN OS Release 36 QA Report

This report says what was tested in the release 36 HDN OS image. It covers the
HDN Linux `7.0.12` patch in QEMU. It does not claim that every physical computer
or production Secure Boot setup has been tested.

## Result

**QEMU result: pass.** The tests found no release-blocking problem in kernel
hardening, the desktop, installation, packages, or sandboxes.

This result does not cover broad physical hardware testing, installation over
FCoE, or a Secure Boot key trusted for production use.

## Image tested

| Field | Value |
| --- | --- |
| ISO | `HDN-OS-Live.x86_64-1.0.0.iso` |
| Size | 2,943,414,272 bytes |
| SHA-256 | `a13c59b1ed27329f7b95d09732695143df2c247645b6a6a8e6e590b3b7e7bad0` |
| Kernel | `kernel-hdn-7.0.12-30.fc42.x86_64` |
| HDN tools | `hdn-tools-7.0.12-23.fc44.x86_64` |
| Release identity | `hdn-release-1-16.fc42.noarch` |
| Desktop integration | `hdn-desktop-1-8.fc44.noarch` |
| Branding | `hdn-branding-1-5.fc42.noarch` |
| Virtual platform | QEMU/KVM q35, x86_64, UEFI Secure Boot, 4 vCPUs, 8 GiB RAM |

The ISO embedded media check and nested ext4 filesystem check passed. All
118,755 files under staged and shipped `/usr` compared byte-for-byte equal.
Release RPM and repository signatures were also verified before image build.

## Security checks

The live image and installed system both reported:

| Check | Result |
| --- | --- |
| Secure Boot | Enabled with the HDN development certificate enrolled in OVMF |
| SELinux | Enforcing |
| HDN policy | Active, generation 1, sealed |
| Policy inventory | 18 profiles and 67 signed object rules |
| Mitigation inventory | 67 of 79 known controls enabled for this product profile |
| Protected system image | `/usr` read-only outside approved transactions |
| Service health | Zero failed systemd units |
| SELinux health | Zero AVC denials after stress and reboot |
| Crash health | Zero coredumps after stress and reboot |
| Kernel health | No kernel-critical journal events |

## Attack checks

The unprivileged live-user suite passed all 20 expected outcomes:

| Surface | Expected result |
| --- | --- |
| Anonymous direct W+X mapping | Denied |
| Writable-to-executable transition | Denied |
| Fixed anonymous executable mapping | Denied |
| Unprivileged BPF | Denied |
| Unprivileged perf events | Denied |
| Unprivileged userfaultfd | Denied |
| Ordinary io_uring use | Allowed |
| Restricted io_uring registration and SQPOLL | Denied |
| Kernel log access | Denied |
| Same-profile ptrace, memory, pidfd, and proc access | Allowed |
| Cross-profile ptrace, memory, pidfd, and proc access | Denied |
| Unprivileged mount and user namespaces | Denied |

A 100-session OpenSSH PTY run completed with 100 successful
privilege-separation handoffs and zero failures. Every session received a PTY;
the release-28 `SCM_RIGHTS` regression is not present in release 30.

## Sandbox And Container Compatibility

| Workload | Result |
| --- | --- |
| Flatpak Calculator from Flathub | Installed and remained running under Bubblewrap |
| Bubblewrap namespace-depth probe | Expected third-level denial; payload still ran |
| Rootless Podman with Alpine | HTTPS request passed |
| Rootless Podman with Node.js 24 | Normal V8 JIT produced the expected result without `--jitless` |
| Container security state | Rootless, cgroups v2, `crun`; no HDN denial or coredump |

The container JIT exception is scoped to the signed container profile. Direct
W+X remains denied in the default and Flatpak profiles.

## Desktop And Application QA

The bundled application matrix passed 24 of 24 launch checks. Five common
desktop applications were then installed through the protected writable-window
transaction:

| Application | Installed version | Launch before reboot | Launch after reboot |
| --- | --- | --- | --- |
| GIMP | 3.2.4 | Pass | Pass |
| VLC | 3.0.23 | Pass | Pass |
| Audacity | 3.7.8 | Pass | Pass |
| Inkscape | 1.4.4 | Pass | Pass |
| Thunderbird | 152.0 | Pass | Pass |

The package transaction refreshed the signed HDN policy and resealed `/usr`,
`/boot`, and the EFI system partition read-only. GIMP and VLC attempted direct
W+X operations that HDN denied; both used compatible fallback paths and stayed
running.

## Module, Service, And Seal Checks

| Check | Result |
| --- | --- |
| Properly signed test module | Loaded and unloaded |
| Mutated signed module | Rejected with `Key was rejected by service` |
| Direct write under sealed `/usr` | Rejected with `EROFS` |
| fwupd metadata refresh | Completed successfully |
| Time synchronization | Active |
| Firewall | Active with only expected desktop services exposed |
| Unexpected listeners | None observed |

## Installation And Persistence

The exact ISO completed the graphical Anaconda workflow with automatic
whole-disk partitioning:

| Partition | Use |
| --- | --- |
| EFI system partition | 629 MiB FAT |
| `/boot` | 2.15 GiB ext4 |
| System volume | Btrfs root and home |

`kernel-install`, `dracut`, GRUB configuration, the BLS entry, and UEFI boot
entry creation all completed successfully. The installed disk then booted
twice with Secure Boot enabled. On the second boot, `/` was writable while
mount-local `/usr`, `/boot`, and `/boot/efi` were read-only, and all test
applications remained installed and launchable.

## Observations And Limits

- Anaconda logged that it could not derive a conventional package name for the
  custom default kernel. The installed `DEFAULTKERNEL=kernel-hdn`, BLS entry,
  signed kernel, initramfs, GRUB configuration, and two disk-only boots were
  all correct. This is nonfatal metadata noise, but should be removed before a
  polished installer release.
- The live installer lacks `/usr/libexec/fcoe/fcoe_edd.sh`. No FCoE hardware
  was present, so installation continued normally. FCoE installation is not
  qualified by this report.
- The Secure Boot result uses an explicitly enrolled development certificate.
  Production media needs a generally trusted signing path or a supported
  enrollment workflow.
- Testing used QEMU/KVM. GPU, suspend/resume, Wi-Fi, Bluetooth, printers,
  docks, unusual storage, and other physical hardware need a separate matrix.
- The 1,111-test kernel harness and this full-system pass complement each
  other; neither is a proof that exploitable defects are absent.
