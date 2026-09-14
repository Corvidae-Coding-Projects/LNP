# Kernel source and release identity

This file answers a simple question: exactly which kernel source produced the
published HDN patch? Use these versions and checksums when reproducing the
release. A similar tree is not the same release.

## Release inputs

HDN kernel development uses normal Git commits in a Linux source tree. The
release-30 source is identified by:

| Input | Identity |
| --- | --- |
| Upstream stable tag | `v7.0.12` |
| Upstream stable commit | `f53879e2e1e2fa053040e734c1ef8f386109a61b` |
| Kernel.org archive SHA-256 | `57edc9a41efc1ca6b797afa8f4a587a30da2af6bca7356eb56e1e1a4ada265da` |
| HDN source commit | `53d6f8a520` |
| HDN source tag | `hdn-v7.0.12-r30` |
| Published source snapshot | `dollspace-gay/hdn-kernel@56a8002ebc` |
| Public patch SHA-256 | `4b3ee5258aa72cca24ad1f11087666e745d131b9d05fa765b8adb9f06b659cf4` |
| Kernel package | `kernel-hdn-7.0.12-30` |
| Policy ABI | `1` |
| HDN OS source tag | `v1.0.0-rc36` |

## Source of truth

The tagged kernel commit is the source used to build HDN. The patch is generated
for people starting with the kernel.org `linux-7.0.12.tar.xz` archive. Package
release numbers track packaging changes. A policy ABI change needs a new ABI
version and matching kernel, tools, policy, and OS packages.

The published `hdn-kernel` repository has two commits: the exact `v7.0.12` tree
and the exact HDN tree. The import commit records the signed upstream commit and
archive checksum. The local development branch may keep the full upstream
history, but its final HDN tree must have the same tree ID as the published one.

Do not edit the generated patch by hand. Make kernel changes as commits in the
source repository, run the needed tests, then rebuild and verify the patch.

## Verification

Set the paths when the kernel Git tree or upstream archive is elsewhere:

```sh
HDN_KERNEL_GIT=/path/to/kernel-git \
HDN_UPSTREAM_ARCHIVE=/path/to/linux-7.0.12.tar.xz \
scripts/verify-release-patch.sh
```

The verifier:

1. verifies the upstream archive and public patch checksums;
2. exports the named HDN source tag;
3. regenerates all unified-diff content from that tag;
4. confirms byte identity with the release-30 patch;
5. runs `patch --dry-run --fuzz=0`; and
6. runs `git apply --check --whitespace=error`.

Release 30 preserves its original destination timestamps in patch headers.
The verifier reads those timestamps as legacy artifact metadata; it does not
reuse patch hunks. A later kernel release may adopt normalized
`SOURCE_DATE_EPOCH` headers under a new documented checksum.
