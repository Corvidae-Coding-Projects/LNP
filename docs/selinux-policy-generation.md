# Custom SELinux policy generation

Implementation decision, 2026-09-13: retain custom policy generation and repair
its scope. This supersedes the earlier product-design instruction to remove
that feature. The user explicitly requested preservation of the capability.

The action remains behind an explicit confirmation and the existing per-action
administrator authentication. The confirmation identifies the alert and explains
that a generated SELinux rule applies to security types, potentially affecting
other files and processes with those types.

The GUI supplies the alert UUID, last-seen date, and exact displayed denial as
expected values. The privileged helper fetches that UUID from setroubleshoot on
the system bus. Only a matching authoritative record can reach `audit2allow`;
caller text alone is never sufficient. Changed or missing alerts fail without
installation. The helper rechecks the selection immediately before installing.

Only one AVC/USER_AVC record is supported per alert event. The current
setroubleshoot API returns the entire audit event, which can contain denials
associated with multiple alerts. Ambiguous events require further review and
are never combined into one permission module. Other records, alerts, and
recent audit-log activity are excluded from compiler input.

The helper generates plain type-enforcement rules with `audit2allow -N`, uses
a private temporary directory, derives a module name from the alert and record,
and logs the installed name. Its success output includes the removal command.
No arbitrary module name or shell command is accepted from the caller.

Validation uses mocked system-bus responses and compiler/installer calls to
cover selected versus unrelated denials, stale records, ambiguous events,
missing alerts, invalid inputs, build failures, and installation failures.
