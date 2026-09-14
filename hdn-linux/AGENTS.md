# Agent Guide: HDN Linux

## Authority and workflow

- Read `README.md` and `docs/SOURCE_CONTROL.md` before changing this subtree.
- This directory publishes a source-derived release artifact, not the editable
  kernel source history. Make functional kernel changes as reviewable commits in
  the named kernel source repository, then regenerate the patch from the exact
  upstream archive and source tag.
- Do not hand-edit `patches/hdn-linux-7.0.12.patch` to implement a kernel change.
- Keep the target upstream version, patch name, source tag, hashes, ABI/package
  identity, and release documentation synchronized.

## Evidence discipline

- Treat `.design/` documents as scoped implementation designs and their
  pipeline JSON as provenance/state for those documents.
- Keep architecture, functional comparison, and measured QA distinct. A finite
  test pass does not prove an unmeasured configuration or future kernel version.
- Report baseline failures, patch regressions, incomplete jobs, and skipped
  environments separately; preserve exact logs outside Git when they are too
  large or machine-specific.
- Run `scripts/verify-release-patch.sh` only with the exact authoritative inputs
  documented in `docs/SOURCE_CONTROL.md`. Do not substitute a nearby kernel tree
  or archive and call the result equivalent.
- Never test kernel installation, module replacement, or boot changes on the
  active system without explicit authorization and a recovery plan.
