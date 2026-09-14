#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
readonly REPO_ROOT
WORKSPACE_ROOT="$(cd -- "${REPO_ROOT}/.." && pwd)"
readonly WORKSPACE_ROOT

readonly PATCH="${HDN_PATCH:-${REPO_ROOT}/patches/hdn-linux-7.0.12.patch}"
readonly KERNEL_GIT="${HDN_KERNEL_GIT:-${WORKSPACE_ROOT}/hdn-kernel-7.0.12}"
readonly UPSTREAM_ARCHIVE="${HDN_UPSTREAM_ARCHIVE:-${WORKSPACE_ROOT}/linux-7.0.12.tar.xz}"
readonly HDN_SOURCE_REF="${HDN_SOURCE_REF:-hdn-v7.0.12-r30}"
readonly EXPECTED_ARCHIVE_SHA256="57edc9a41efc1ca6b797afa8f4a587a30da2af6bca7356eb56e1e1a4ada265da"
readonly EXPECTED_PATCH_SHA256="4b3ee5258aa72cca24ad1f11087666e745d131b9d05fa765b8adb9f06b659cf4"
# GNU diff includes numeric timezone offsets in unified-patch headers. The
# canonical release-30 patch was generated in America/Chicago: ordinary files
# are dated during daylight time, while missing-file epoch headers use standard
# time. Pin the geographic zone independently of the release builder.
readonly PATCH_TZ="America/Chicago"

for command in awk cmp diff git patch sha256sum tar touch; do
	command -v "${command}" >/dev/null ||
		{
			printf 'missing required command: %s\n' "${command}" >&2
			exit 2
		}
done

[[ -f "${PATCH}" ]] || {
	printf 'patch not found: %s\n' "${PATCH}" >&2
	exit 2
}
[[ -f "${UPSTREAM_ARCHIVE}" ]] || {
	printf 'upstream archive not found: %s\n' "${UPSTREAM_ARCHIVE}" >&2
	exit 2
}
git -C "${KERNEL_GIT}" rev-parse --verify "${HDN_SOURCE_REF}^{commit}" >/dev/null

printf '%s  %s\n' "${EXPECTED_ARCHIVE_SHA256}" "${UPSTREAM_ARCHIVE}" |
	sha256sum --check --status
printf '%s  %s\n' "${EXPECTED_PATCH_SHA256}" "${PATCH}" |
	sha256sum --check --status

workdir="$(mktemp -d)"
if [[ "${HDN_KEEP_WORKDIR:-0}" == "1" ]]; then
	printf 'keeping verification work directory: %s\n' "${workdir}"
else
	trap 'rm -rf -- "${workdir}"' EXIT
fi

TZ="${PATCH_TZ}" tar -xJf "${UPSTREAM_ARCHIVE}" -C "${workdir}"
mkdir "${workdir}/linux-7.0.12-hdn"
TZ="${PATCH_TZ}" git -C "${KERNEL_GIT}" archive "${HDN_SOURCE_REF}" |
	tar -xf - -C "${workdir}/linux-7.0.12-hdn"

# The reconstructed source Git predates this verifier and its broad `.*`
# ignore rule omitted the unchanged Documentation rename map.  Carry it
# through only after proving that the working-tree copy still matches the
# upstream archive, so this compatibility fix cannot hide content drift.
readonly RENAME_MAP="Documentation/.renames.txt"
cmp --silent \
	"${workdir}/linux-7.0.12/${RENAME_MAP}" \
	"${KERNEL_GIT}/${RENAME_MAP}" || {
	printf 'reconstruction passthrough differs from upstream: %s\n' \
		"${RENAME_MAP}" >&2
	exit 1
}
cp --preserve=mode,timestamps \
	"${KERNEL_GIT}/${RENAME_MAP}" \
	"${workdir}/linux-7.0.12-hdn/${RENAME_MAP}"

# The release-30 patch predates deterministic header timestamps. Reuse only
# its destination timestamps while regenerating every content hunk from Git.
awk -F '\t' '
	/^\+\+\+ linux-7\.0\.12-hdn\// {
		sub(/^\+\+\+ linux-7\.0\.12-hdn\//, "", $1)
		print $1 "\t" $2
	}
' "${PATCH}" |
	while IFS=$'\t' read -r relative_path timestamp; do
		TZ="${PATCH_TZ}" touch -h -d "${timestamp}" \
			"${workdir}/linux-7.0.12-hdn/${relative_path}"
	done

generated_patch="${workdir}/hdn-linux-7.0.12.patch"
set +e
(
	cd "${workdir}"
	LC_ALL=C TZ="${PATCH_TZ}" diff -ruN \
		'--exclude=.git' \
		'--exclude=.config' \
		'--exclude=*.o' \
		'--exclude=*.cmd' \
		'--exclude=.tmp_*' \
		'--exclude=certs/signing_key*' \
		linux-7.0.12 linux-7.0.12-hdn
) >"${generated_patch}"
diff_status=$?
set -e
[[ "${diff_status}" -eq 1 ]] || {
	printf 'diff returned unexpected status %d\n' "${diff_status}" >&2
	exit 1
}

cmp --silent "${PATCH}" "${generated_patch}" || {
	printf 'generated patch differs from %s\n' "${PATCH}" >&2
	sha256sum "${PATCH}" "${generated_patch}" >&2
	exit 1
}

patch --batch --dry-run --fuzz=0 -p1 \
	-d "${workdir}/linux-7.0.12" <"${generated_patch}" >/dev/null
git -C "${workdir}/linux-7.0.12" init --quiet
git -C "${workdir}/linux-7.0.12" apply \
	--check --whitespace=error "${generated_patch}"

printf 'PASS source-ref=%s patch-sha256=%s\n' \
	"${HDN_SOURCE_REF}" "${EXPECTED_PATCH_SHA256}"
