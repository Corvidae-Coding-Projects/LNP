# Agent Guide: HDN Linux

## Source and workflow

- Read `README.md` and `docs/SOURCE_CONTROL.md` before changing this subtree.
- This directory contains the published patch, not the kernel's editable Git
  history. Make kernel changes in the source repository named in
  `docs/SOURCE_CONTROL.md`, then rebuild the patch from the listed archive and
  source tag.
- Do not hand-edit `patches/hdn-linux-7.0.12.patch` to implement a kernel change.
- When the release changes, update the kernel version, patch name, source tag,
  hashes, package details, ABI details, and release notes together.

## Test records

- `.design/` holds plans for specific changes. The matching JSON files record
  the state of each plan.
- Keep architecture, functional comparison, and measured QA distinct. A finite
  test pass does not prove an unmeasured configuration or future kernel version.
- Report existing upstream failures, HDN failures, unfinished jobs, and skipped
  test systems separately. Keep exact logs outside Git when they are too large
  or tied to one machine.
- Run `scripts/verify-release-patch.sh` only with the exact inputs listed in
  `docs/SOURCE_CONTROL.md`. A nearby kernel tree or archive is not equivalent.
- Never test kernel installation, module replacement, or boot changes on the
  active system without explicit authorization and a recovery plan.
