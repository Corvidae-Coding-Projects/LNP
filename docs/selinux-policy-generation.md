# Custom SELinux policy generation

Decision from 2026-09-13: keep custom policy generation, but limit it to one
specific and verified SELinux alert. This replaces the earlier plan to remove
the feature.

The user must confirm the change and approve it as an administrator. The app
names the alert and explains that SELinux rules apply to security types. This
means a rule may affect other files or programs that use the same types.

The app sends the alert ID, date, and the exact denial it showed. The
administrator helper loads that alert again from setroubleshoot. It sends data
to `audit2allow` only when the system record matches exactly. A missing or
changed alert stops the process. The helper checks again before installation.

One alert event must contain exactly one supported denial. Setroubleshoot can
return an event with denials from more than one alert. The helper rejects those
events instead of guessing or combining them. It does not use other alerts or
recent audit-log entries.

The helper creates a basic type-enforcement rule with `audit2allow -N`. It uses
a private temporary folder and builds the module name from the checked alert.
It records the installed name and prints the command needed to remove it. The
app cannot provide its own module name or shell command.

Tests use fake system-bus, compiler, and installer calls. They cover unrelated
or old denials, unclear events, missing alerts, bad input, build errors, and
installation errors without changing the host's SELinux policy.
