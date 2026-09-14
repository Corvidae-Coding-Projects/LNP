# Kernel Source And Release Identity

## Authoritative Inputs

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

## Source Of Truth

The tagged kernel source commit is the implementation source of truth. The
unified patch is a generated release interface for users who start from the
kernel.org `linux-7.0.12.tar.xz` archive. Package release numbers identify
packaging revisions of that source. Policy ABI changes require an explicit ABI
version change and coordinated kernel, tools, policy, and OS package releases.

The published `hdn-kernel` repository uses a compact two-commit snapshot chain:
the exact `v7.0.12` tree followed by the exact HDN tree. Its upstream import
message records the signed stable commit and archive checksum. This avoids
republishing the kernel's full historical object graph while preserving normal
line-level history for every HDN change. The local development branch may
retain the full signed-tag ancestry; the published and local HDN tree IDs are
identical.

Do not make functional edits directly in the generated patch. Make them as
reviewable commits on the kernel source branch, run the focused and full
hardening suites, then regenerate and verify the public artifact.

## Verification

Set paths explicitly when the kernel Git tree or upstream archive is not next
to this repository:

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
