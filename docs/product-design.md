# Linux for Normal People: Product and Safety Design Specification

| Field | Value |
| --- | --- |
| Status | Proposed product contract; implementation is not yet conformant |
| Version | 1.0 |
| Date | 2026-08-08 |
| Audience | LNP maintainers, KDE/polkit reviewers, security reviewers, packagers, researchers, and testers |
| Scope | Product doctrine, end-to-end experience, authorization threat model v2, usability research plan, core journey matrix, accessibility audit, release gates, and delivery sequence |
| Replaces | The architectural conclusions in `docs/consent-design.html`; that document remains useful as design history only |

## Document map

- **Part I — Product doctrine:** north star, people, jobs, principles, platform
  lessons, experience architecture, content, measures, and release gates.
- **Part II — Authorization and security architecture v2:** threat model,
  invariants, operation risk ladder, present-safe polkit path, future trusted
  intent design, prompt behavior, accessibility/security boundary, and tests.
- **Part III — Core-journey research and test matrix:** recruitment, ethics,
  fixtures, facilitator protocol, scoring, 22 end-to-end journeys, fault
  injection, analysis, and reporting.
- **Part IV — Accessibility audit and conformance plan:** baseline, current-code
  findings, requirements by modality, audit procedure, and stable-release gate.
- **Part V — Delivery plan and traceability:** phased work, current repository
  evidence, required decision records, definition of complete, and sources.

## Executive decision

LNP is not a prettier Fedora spin and it is not a Linux tutorial. It is a
consumer desktop whose implementation happens to use Fedora and KDE Plasma.
Its north star is:

> LNP lets a person with no Linux-specific knowledge accomplish everyday
> work, maintenance, troubleshooting, and recovery safely and confidently,
> without needing a terminal or understanding how the operating system is
> assembled.

The terminal is a canary, not the literal boundary of failure. A graphical
dialog that asks someone to understand SELinux types, repositories, package
formats, services, or kernel modules has failed just as thoroughly. Conversely,
a command prompt deliberately offered to a remote helper in an explicitly
labelled support path does not make the ordinary experience a failure.

“Normal people” is brand shorthand for people who should not need a professional
relationship with their operating system. It never means one kind of body,
mind, literacy, language, age, or ability. If the product works only for a
sighted, dexterous, confident mouse user, it has contradicted its own name.

This specification makes four decisions:

1. **Design around human jobs, not Linux components.** The primary navigation,
   wording, status, and recovery paths describe what a person is trying to do.
2. **Make safety architectural.** Prefer bounded operations, safe defaults,
   automatic prevention, snapshots, undo, and least privilege over warnings.
3. **Do not pretend that a click is authentication.** LNP will use today's
   polkit honestly while pursuing an upstream, trusted *intent authorization*
   path for the narrow class of changes where a protected human gesture is
   sufficient.
4. **Treat accessibility and research as release requirements.** A feature is
   not complete when it only works with a mouse, at default scale, for its
   author. Every critical journey must pass automated inspection, expert
   assistive-technology testing, and repeated tests with representative people.

The practical consequence for the present repository is immediate: do not
implement the legacy `auth_consent` sketch as written. First split broad root
helpers into typed, independently authorized operations; remove retained
authorization; remove the one-click custom SELinux-module path; make the dock
operable and exposed through AT-SPI; and put `lnp-selinux` in the default
installation. Only then prototype trusted intent authorization upstream.

---

## Part I — Product doctrine

### 1. Purpose of this doctrine

This part is the decision filter for LNP. It is intentionally normative. The
words **must**, **should**, and **may** mean required, recommended unless there
is recorded evidence for an exception, and optional respectively.

When a feature request, upstream convention, security control, or implementation
shortcut conflicts with this doctrine, the maintainer must do one of three
things:

- change the design to conform;
- document the exception, affected users, risk, evidence, and removal plan in a
  decision record; or
- decline the feature.

“Other Linux desktops do this” is context, not evidence. “Windows/macOS users
expect this” is a hypothesis until tested in LNP's context of use. Familiarity
is valuable, but LNP copies a convention only when it improves transfer of
learning without importing its failure modes.

### 2. Product promise and boundaries

#### 2.1 The promise

For supported hardware and documented use cases, a person must be able to:

- discover and launch apps, switch between them, and find a lost window;
- connect common networks, Bluetooth devices, displays, printers, scanners,
  cameras, and removable storage;
- create, find, move, share, back up, and recover personal files;
- install, update, remove, and understand the provenance of applications;
- play common media and install a required hardware driver;
- understand system status without monitoring it;
- respond safely to a permission or authorization request;
- recover from a failed update or failed graphical login;
- get useful help or invite a trusted helper without exposing a terminal as the
  first-line solution; and
- enable and use accessibility features independently.

The experience must remain safe when the person reads imperfectly, acts under
time pressure, has learned to dismiss prompts, or does not know which subsystem
owns a problem. Those are normal operating conditions, not “user error.”

#### 2.2 Supported context of use

The initial product context is a personal or household x86-64 laptop or desktop
running the supported Fedora KDE base, using a local graphical Wayland session,
with Btrfs root and a separate home subvolume. Supported input includes
keyboard, pointer, touchpad, touchscreen where hardware permits, and common
assistive technologies available in Fedora/KDE.

The primary account created during setup is the **owner**: normally a member of
the administrative group but always deprivileged during ordinary work. Other
household accounts may be standard users. The product must not assume that the
person at the screen knows the owner credential.

#### 2.3 Explicit non-goals for the first product boundary

LNP does not initially promise:

- to expose every Fedora or KDE option;
- to make arbitrary third-party scripts, repositories, kernel modules, or
  custom SELinux policy safe;
- to turn an already compromised user account into a safe place for personal
  files;
- to replace enterprise device management or a professional Linux workstation;
- to preserve every unsupported Plasma customization across a major layout
  migration;
- to conceal that a consequential action has consequences; or
- to prevent an owner from deliberately entering a clearly labelled expert or
  support path.

Power-user capability may remain available through the underlying system. It
must not leak into the language or structure of the normal path, and it must
not weaken the normal path merely to make unsupported customization easier.

### 3. People LNP is for

Personas are not demographic stereotypes. These are behavior and context
profiles used for recruitment and design reviews. One participant may match
several profiles.

| Profile | Relevant characteristics | LNP obligation |
| --- | --- | --- |
| Everyday owner | Uses a computer for web, media, documents, communication, and light games; knows Windows or macOS conventions | Make transfer of learning work; keep system administration out of the task |
| Low-confidence user | Fears breaking things, reads warnings literally, may abandon rather than explore | Make safe exploration visibly safe; provide cancel, undo, and truthful reassurance |
| Learned click-through user | Has been trained by noisy software to accept prompts reflexively | Minimize prompts; make risky requests discriminable; measure comprehension rather than clicks |
| Older or intermittent user | May have reduced vision, dexterity, working memory, or long gaps between use | Favor recognition, persistent labels, forgiving targets, and recoverable state over memorized gestures |
| Disabled user | May use Orca, magnification, high contrast, keyboard-only control, switch access, speech control, or reduced motion | Provide semantic equivalence and independent completion, including in trusted and recovery UI |
| Household standard user | Can use the computer but cannot authorize owner-level changes | Explain who can approve, preserve the task, and allow approval without sharing a reusable password with the requesting app |
| Trusted helper | Family member, friend, repair shop, or support worker helping locally or remotely | Offer consented, time-bounded diagnostics and remote assistance; reveal technical detail progressively |
| Interrupted user | Is low on battery, offline, in a meeting, or responding to an unexpected failure | Defer noncritical work, preserve context, state what happens next, and never manufacture urgency |

The team must not use its own members as a proxy for these people. Familiarity
with Linux changes search strategies, terminology, risk perception, and the
willingness to open a terminal.

### 4. Human needs and jobs

Features are accepted against jobs, not component ownership. The durable jobs
are:

1. **Start and resume.** “Get me back to what I was doing.”
2. **Find and act.** “Help me find my app, file, window, setting, or device.”
3. **Connect.** “Make this network, display, headset, printer, or storage work.”
4. **Add capability.** “Let me use this app, media format, or hardware.”
5. **Know whether all is well.** “Tell me only when I need to act.”
6. **Decide safely.** “Tell me what is asking, what will change, and what my
   options mean.”
7. **Undo and recover.** “Put me back in a working state without sacrificing my
   personal files.”
8. **Get help.** “Help me describe or share the problem without becoming the
   technician.”
9. **Adapt the computer.** “Make it work with my body, senses, language, and
   preferences.”

A design review begins by naming the job and the context. If the proposed UI is
named after a daemon, package, policy engine, filesystem, or implementation
layer, that is a strong signal that the team has not finished designing it.

### 5. Product principles

#### 5.1 Obvious first, powerful when needed

The primary action and the next state must be visible. Optional detail may be
progressively disclosed under a label that describes its contents—“Packages
being installed” or “Technical details,” not merely “Advanced.” This follows
KDE's own guidance on [progressive disclosure](https://develop.kde.org/hig/powerful_when_needed/).

Primary workflows must not depend on hover, right click, a hidden gesture, a
keyboard shortcut, or memorizing where a setting lives. Those may accelerate a
workflow but cannot be its only route.

#### 5.2 Recognition over recall

Use familiar nouns, visible state, recent items, searchable labels, and stable
locations. Preserve a person's context after an interruption. Do not require
them to remember an error code, command, repository name, or what changed before
a restart.

#### 5.3 Preserve agency and prior choices

Defaults apply only until a person makes a choice. Updates and layout migrations
must not silently overwrite that choice. Where LNP cannot distinguish a choice
from incidental state, it must back up and offer a clear restore path before
changing it.

The existing XDG defaults cascade embodies this principle. Direct writes to a
user configuration need stronger justification, a per-setting “unset” check,
and an undo path.

#### 5.4 Prevent before warning

The preferred order is:

1. make the dangerous state unrepresentable;
2. choose a safe default;
3. constrain scope and privilege;
4. make the operation transactional or reversible;
5. detect and repair automatically;
6. ask a contextual question; and only then
7. display a warning.

This is the product interpretation of NIST's human-centered cybersecurity goal:
make the right action easy, the wrong action hard, and recovery easy when the
wrong action occurs ([NIST Human-Centered Cybersecurity](https://csrc.nist.gov/Projects/human-centered-cybersecurity/about)).

#### 5.5 Interrupt only for an action that matters now

Alerts are a scarce channel. Apple similarly recommends using alerts sparingly
because they interrupt the current task and avoiding them for information-only
or undoable actions ([Apple alerts guidance](https://developer.apple.com/design/human-interface-guidelines/alerts)).

LNP must not interrupt for:

- successful background work;
- a condition the system will retry or repair;
- technical churn with no user action;
- an old event discovered at login; or
- a recommendation that can wait in Computer Care.

Every notification must have one of three outcomes: a useful action, a safe
deferral, or a durable place where the person can find it later. Repetition
must collapse into one item with a count. Rate limiting is a last defense, not
permission to generate low-value events.

#### 5.6 Explain consequences, not mechanisms

The first layer says:

- what happened or what is being proposed;
- what it affects;
- whether anything has already changed;
- what the recommended next action is; and
- whether it is reversible.

Technical nouns belong in a secondary detail view and in an exportable support
record. “The security system stopped Photos from reading this folder” is the
normal message. “SELinux denied `read` to `scontext`…” belongs in details.

#### 5.7 Honest calm

Calm wording is not blanket reassurance. LNP must not say “your files are safe”
unless it can establish that for the described failure, or “nothing is broken”
when a task was blocked. It must not say “updates cannot hurt you”; snapshots
reduce risk but do not cover firmware, user data migrations, hardware failure,
or every boot path.

Use precise claims such as “The update did not finish. The previous system
snapshot is still available, and your personal files were not part of this
change.”

#### 5.8 Accessibility is functional correctness

If a control has no accessible name, a workflow has no keyboard route, text is
clipped at 200% scale, status is color-only, or a security decision cannot be
made with the person's assistive technology, the feature is broken. An
“accessible alternative” buried in a terminal is not equivalent.

#### 5.9 Security decisions are system decisions

An application may request an operation, but it must not author the trusted
prompt, decide its own display name, provide the consequence text, receive a
password, or expand the authorized scope after approval. Those are jobs for
root-owned policy and the trusted system path described in Part II.

#### 5.10 Recovery is part of the primary experience

Backup, undo, restart safety, and failed-login recovery are designed and tested
with the feature that needs them. They are not deferred to documentation.
System recovery must preserve home data by construction, name the exact system
state being restored in plain language, and keep the replaced state long enough
to reverse the recovery itself.

#### 5.11 What to learn from Windows, macOS, and KDE

LNP should use familiarity as a bridge, not create a visual impersonation of
another platform.

From **Windows**, adopt visible app launching and running state, broad hardware
expectations, searchable settings, predictable dialog verbs, deprivileged
ordinary execution, and explicit elevation. Avoid fragmented old/new settings,
generic “make changes” prompts, standing administrator behavior, bundled
recommendations, and compatibility decisions that silently weaken a security
boundary.

From **macOS**, adopt system-wide coherence, sparse interruptions, strong
separation between app UI and system authorization, just-in-time resource
permissions, hardware/software integration where LNP can actually support it,
and accessibility designed at platform level. Avoid hiding consequential state
for visual simplicity, unexplained gestures, opaque provenance, and treating a
polished surface as proof that a decision is understood.

From **KDE**, adopt native platform components, customization that preserves
valid user workflows, explicit keyboard access, and progressive disclosure.
Avoid exposing the full component graph merely because the framework makes it
configurable. LNP should contribute generic fixes upstream rather than maintain
a permanently forked shell or security stack whenever feasible.

Across all three, copy the *reason* a convention works: stable placement,
recognizable state, system mediation, or a real trust boundary. Do not copy the
pixels while omitting that reason.

#### 5.12 Evidence discipline

The familiar usability heuristics—system-status visibility, match with the real
world, user control, consistency, error prevention, recognition over recall,
and recovery—are design prompts, not proof. A heuristic review finds plausible
problems; observation tests whether specified people achieve specified goals in
context. Security review asks whether the claimed invariant survives an
adversary. Accessibility review asks whether equivalent perception and action
exists. LNP requires all three because success in one can conceal failure in
another.

### 6. Experience architecture

#### 6.1 Stable places

LNP should present a small, stable mental model:

| Place | Person's question | Implementation direction |
| --- | --- | --- |
| Launcher/search | “Where is it?” | Search apps, common settings, help topics, recent files, and actions with human labels |
| Dock/window switcher | “What is open and how do I get back?” | Running state, launch/focus/minimize, keyboard and screen-reader equivalents |
| System Settings | “How do I want the computer to behave?” | Use upstream KDE pages where coherent; provide task-oriented entry points rather than duplicating settings |
| Software | “How do I add or remove an app?” | Prefer sandboxed/curated apps; show source and trust only when it changes the decision |
| Computer Care | “Does anything need me?” | Evolve the one-time Welcome app into a durable status, maintenance, history, and undo surface |
| Help & Support | “How do I solve or share this?” | Contextual articles, diagnostic bundle, recovery history, and consented remote help |
| Recovery | “How do I get a working desktop back?” | Graphical where possible; accessible local fallback before a helper-only shell |

“Computer Care” is a working label to test, not a fixed brand decision. It
should replace the current pattern of putting media setup, driver installation,
disk cleaning, and unrelated security configuration into a first-run wizard.
Onboarding may point to it, but routine maintenance must remain discoverable
after onboarding.

#### 6.2 Onboarding

Onboarding must earn every page. It should:

- orient the person to the launcher, dock, status area, settings, and Help;
- detect required hardware or media work and show only relevant actions;
- offer to preserve or restore the previous desktop without implying the new
  choice is permanent;
- invite essential accessibility setup before visually dense content;
- allow “Not now” without nagging on every login; and
- finish in under three minutes when no intervention is required.

It must not ask the person to perform general maintenance “because they are
here,” introduce technical vocabulary pre-emptively, or bundle several root
changes behind one credential. Close means “not now”; it does not mean consent,
completion, or permanent refusal.

#### 6.3 Search and navigation

Search is the safety net for information architecture, not a substitute for
it. Search terms must include ordinary synonyms (“wifi,” “internet,” “screen,”
“monitor,” “apps,” “programs,” “free space”) and may index the implementation
term only as an alias. Results should show the destination and current state
when useful.

The dock, launcher, system tray, settings, notifications, and recovery surfaces
must share:

- the same app names and icons;
- the same words for owner, administrator, restart, remove, and restore;
- predictable Back, Cancel, Close, and Escape behavior;
- visible focus and keyboard order matching visual order; and
- native locale, writing direction, font, scaling, contrast, and motion
  preferences.

#### 6.4 Status and background work

Background work has four presentation states:

| State | Presentation |
| --- | --- |
| No action needed | Silent; durable history if diagnostically useful |
| In progress after a user action | Inline progress with task name, safe-close behavior, and cancellation only when cancellation is real |
| Deferred or retrying | Inline state with next retry and optional “Try now” |
| User action needed | One notification leading to a persistent, contextual action surface |

Never use an indeterminate spinner for a step whose progress can be measured.
Never fake percent completion. Closing a progress window must not silently kill
an operation that must finish; it should move the task to Computer Care and
notify once on completion or actionable failure.

#### 6.5 Error message contract

Every user-facing error must be generated from structured state, not by
displaying helper stdout or an exception directly. It uses this template:

1. **Title:** concrete outcome—“The NVIDIA driver wasn't installed.”
2. **What remains true:** “Your current graphics driver is still working.”
3. **Likely cause, only when known:** “The download stopped when the internet
   connection changed.”
4. **Primary action:** a realistic verb—“Try again.”
5. **Secondary action:** “Do this later” or “Get help,” when useful.
6. **Details:** time, operation ID, exact packages, logs, and copy/export
   controls for a helper.

Do not blame the person, present raw codes as the title, say “contact your
administrator” on a personal computer, or recommend a command. Microsoft's
writing guidance likewise emphasizes a warm, concise explanation, what happens
next, and a solution the person can perform ([Windows writing style](https://learn.microsoft.com/en-us/windows/apps/design/style/writing-style)).

#### 6.6 Destructive action and undo contract

For a common, easily reversible action, act immediately and provide Undo. For
an unusual or incompletely reversible action, confirm at the point of action.
The confirmation names the object and consequence; the primary button names the
operation, never “OK” or “Yes.”

An undo record must include the operation, parameters, actor, time, prior state,
expiry, and result. Undo must be tested after partial failure and restart, not
only on the happy path.

#### 6.7 Help and human support

Help begins in context. “Get help” must be available from each actionable error
and Computer Care. It should produce a plain summary and, with separate consent,
an exportable diagnostic bundle that:

- previews every category of collected data;
- excludes file contents, browser data, credentials, encryption keys, and
  unrelated journal history by default;
- redacts usernames, home paths, network identifiers, and serial numbers where
  they are not required;
- records the bundle schema and collection time; and
- can be deleted from the same UI.

Remote assistance must be initiated by the person, visibly active, time-bounded,
revocable at any moment, and separated into view, input, clipboard, file
transfer, and privileged-action capabilities. A helper cannot approve a local
trusted prompt invisibly. Accessible remote assistance is addressed in Part IV.

### 7. Content vocabulary

Use the left column in normal UI. The right column is allowed in technical
details and developer logs.

| Normal UI | Technical detail only |
| --- | --- |
| Apps / Software | packages, RPM, Flatpak, repository |
| Built-in security system | SELinux, AVC, policy type, boolean |
| Background service | daemon, systemd unit |
| Saved system state | Btrfs subvolume snapshot |
| Owner approval | polkit authentication, wheel identity |
| Administrator password | PAM credential |
| Software source | RPM Fusion repository |
| Graphics driver | akmod, kernel module |
| Restart | reboot |
| Personal files | `/home` subvolume |

Rules:

- Use sentence case and active voice.
- Use “you” for the person's choice, not for blame; use “the computer” for
  automated behavior.
- Prefer specific verbs: Install, Remove, Restore, Share, Allow once, Keep
  blocked.
- Avoid “simply,” “just,” “obviously,” “invalid,” and “failed” without an
  object.
- Do not communicate severity by color, icon, or sound alone.
- Do not promise legal safety or universal hardware support.
- Localize meaning, not string fragments; allow at least 200% text expansion
  and long translated button labels.

### 8. Product success measures

ISO 9241-11 frames usability in terms of specified users achieving specified
goals with effectiveness, efficiency, and satisfaction in a specified context
of use ([ISO 9241-11:2018](https://www.iso.org/standard/63500.html)). LNP adds
safety, accessibility, recovery, and interruption cost because a desktop can be
fast and pleasant while teaching unsafe approval behavior or excluding people.

#### 8.1 North-star measure

**Safe independent journey completion:** the proportion of representative
participants who complete a defined everyday or recovery journey without a
terminal, external instructions, unsafe authorization, or facilitator rescue,
while preserving the intended security invariant.

This is a suite of journey measures, not one vanity percentage. Results must be
segmented by journey, confidence level, account role, input method, assistive
technology, and relevant hardware. An average must never hide a critical group
or task failure.

#### 8.2 Required measures

| Dimension | Measure | Product target before stable release |
| --- | --- | --- |
| Effectiveness | Unassisted completion of each critical normal journey | At least 90% in a benchmark sample; no repeated blocker in iterative rounds |
| Terminal dependency | Critical journeys requiring terminal, copied command, or Linux-specific concept | 0 |
| Safety | Unsafe approval in adversarial prompt scenarios | 0 in release rounds; benchmark and publish confidence intervals at scale |
| Comprehension | Can state requester, action, affected scope, and whether it is reversible after a consequential prompt | At least 80% per field; 100% for scope in Tier 3 tests before release |
| Recovery | Successful return to working desktop with personal files unchanged in injected update/login failures | 100% of supported fixtures |
| Efficiency | Time and wrong turns versus the previous LNP release and current Fedora KDE baseline | No critical regression; task-specific targets set after baseline |
| Accessibility | Independent completion of every critical journey using each applicable supported modality | 100%; no critical or high open finding |
| Interruption | Nonactionable or duplicate notifications during a seven-day scripted workload | 0 |
| Reliability | Operation reaches an honest terminal state after network loss, cancel, crash, and restart | 100% of fault-injection cases |
| Satisfaction | Single Ease Question after each task plus confidence interview | Median at least 5/7; never used to override observed failure |

Early rounds of four to eight people are for discovering failure modes, not
claiming statistical proof. Numeric stable-release targets require a larger,
pre-registered benchmark sample or defensible production evidence with privacy
protections.

#### 8.3 Privacy-preserving product evidence

LNP must work without cloud telemetry. Development images may offer explicit,
off-by-default research logging that records structured journey events without
content: action class, duration, cancellation, error category, recovery, and
accessibility mode if the participant chooses to disclose it. The UI must show
what is recorded and support export/delete.

Stable releases should prefer local health history and opt-in problem reports.
No report may include filenames, command history, typed text, window titles,
clipboard contents, SSIDs, account names, or persistent device identifiers by
default. “Anonymous” is not an acceptable claim without a documented
re-identification analysis.

### 9. Product release gates

A feature affecting a critical journey cannot ship as stable until all of the
following are true:

- the user need and supported context are named;
- the primary route is visible and terminal-free;
- system and security boundaries are documented;
- user-facing states and copy exist for success, empty, offline, denied,
  cancelled, partial failure, restart, and retry;
- changes are least-privileged and reversible where technically possible;
- all controls have semantic roles, names, state, and keyboard operation;
- automated tests cover validation and state transitions;
- manual accessibility checks in Part IV pass;
- at least one representative research round tested the end-to-end behavior,
  including failure rather than screenshots alone;
- no unresolved critical or high research/security/accessibility finding
  remains; and
- the support and recovery path has been exercised from a clean supported
  installation.

The release owner signs a short evidence record linking tests, research
findings, audit results, and accepted exceptions. Absence of a bug report is
not evidence of completion.

---

## Part II — Authorization and security architecture v2

### 10. Security objective

LNP must let a deprivileged person request narrow system changes without
turning every desktop process into an administrator, teaching indiscriminate
approval, exposing reusable credentials to applications, or granting authority
beyond the exact understood action.

The design distinguishes three concepts:

- **Identity authentication:** evidence of who is approving, such as an owner
  password, fingerprint, hardware-backed credential, or recovery credential.
- **Intent confirmation:** evidence that a human in the active local session
  deliberately approved this exact operation now.
- **Authority:** the system's decision that this authenticated identity or
  confirmed intent may cause this exact operation under current policy.

A click can provide intent only if the click and displayed meaning are protected
from the requesting process. It is never evidence that an identity
authenticated. A cookie or request handle correlates messages; it is not, by
itself, authority.

### 11. Lessons adopted from other systems

#### 11.1 Windows

Classic UAC gives an administrator a filtered token for normal work and moves
elevation prompts to Secure Desktop by default. Standard users receive a
credential prompt rather than an administrator's consent prompt. The valuable
idea is not the dimmed screen: it is separation between the requesting desktop
and the decision surface ([Microsoft, “How UAC works”](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/how-it-works)).

The newer Windows 11 Administrator Protection design is a stronger north-star
analogy: the user remains deprivileged, verifies each admin operation with
Windows Hello, and Windows creates a profile-separated, isolated, disposable
admin token for only the requesting process. Microsoft explicitly describes the
new architecture as a security boundary, although as of this document it
remains preview/gradual-rollout functionality
([Administrator Protection](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/administrator-protection/)).

LNP adopts deprivileged normal work, just-in-time scope, per-operation approval,
isolated mechanism execution, and auditable outcomes. It does not copy
auto-elevation, a generic “allow this app to make changes” prompt, or authority
retention across unrelated parameters.

#### 11.2 macOS

macOS Authorization Services places policy and credential handling in system
services. The requesting app can obtain authorization without receiving the
username/password, and the authentication method can evolve (for example to
Touch ID) without changing the app
([Apple Authorization Services](https://developer.apple.com/documentation/security/authorization-services)).

LNP adopts system-mediated credentials, policy-named rights, and replaceable
authentication methods. For user resources such as files, cameras, and screen
sharing, LNP favors just-in-time portal-style grants over root elevation.

#### 11.3 Linux building blocks

Polkit is an authority for mechanisms servicing untrusted subjects. Its current
implicit outcomes are `no`, `yes`, `auth_self`, `auth_admin`, and retained
variants. Its agent interface is explicitly an *authentication* interface. It
does not currently contain an honest consent-only result. Polkit also warns that
retained authorization for the same action and subject may succeed while
details differ
([polkit reference manual](https://polkit.pages.freedesktop.org/polkit/polkit.8.html)).

XDG Desktop Portal provides a useful interaction model: a request returns a
caller-bound object handle, stays alive for the user interaction, produces one
structured response, and can be closed. The supplied handle token avoids races
and naming collisions; it is not treated as the authorization itself
([portal request model](https://flatpak.github.io/xdg-desktop-portal/docs/requests.html)).

Wayland prevents ordinary clients from freely reading or injecting one
another's input and display contents, but that alone does not make an ordinary
client window trusted. KWin must explicitly own and defend any surface claimed
as a trusted intent path, and that claim requires upstream security review.

### 12. Assets and invariants

The architecture protects:

- system integrity, bootability, security policy, trusted software sources, and
  administrative configuration;
- the confidentiality of reusable credentials and authentication factors;
- the semantic integrity of the decision—what the person saw is exactly what
  the mechanism may do;
- the integrity of audit and undo records;
- separation between household standard users and owners;
- availability of an escape/cancel path; and
- the person's attention as a finite safety resource.

The following invariants are non-negotiable:

**S1. Exact operation binding.** Approval is bound to one action ID, normalized
typed parameters, requester identity, local session, and expiry. Changing any
one creates a new request.

**S2. Narrow mechanism.** The privileged process accepts a closed operation set
and independently validates every parameter after authorization. It never
executes caller-provided shell, arbitrary executable paths, package URLs,
commands, environment, or policy text.

**S3. Trusted meaning.** The action title, consequence, risk tier, parameter
rendering rules, and undo promise come from root-owned metadata and mechanism
state, never caller strings.

**S4. Trusted actor.** The broker obtains UID, PID, process start time, security
label, and bus identity from the kernel/message bus. It does not accept those as
method arguments.

**S5. No ambient elevation.** The desktop app stays unprivileged. The broker
executes only the narrow operation and discards request state immediately.

**S6. No cross-operation reuse.** A successful decision never authorizes a
different action or different parameters. LNP does not use `auth_admin_keep` or
invent `auth_consent_keep`.

**S7. One terminal result.** Each request becomes success, denied, cancelled,
expired, superseded, or failed exactly once. Client exit and UI dismissal fail
closed.

**S8. Audit without secrets.** The system records enough to answer who requested
what class of change, when, with what normalized scope, result, and undo ID; it
does not log credentials or sensitive file content.

**S9. Accessible equivalence.** Trusted decisions remain independently usable
with supported assistive technologies. Accessibility does not grant a separate,
weaker approval channel.

**S10. Denial is safe.** Denying, timing out, losing focus, locking the session,
switching users, or disconnecting a remote helper leaves the system unchanged.

### 13. Threat model

#### 13.1 Adversaries in scope

| Adversary/cause | Capability | Required defense |
| --- | --- | --- |
| Ordinary buggy app | Calls an operation incorrectly, repeats it, crashes, or changes state mid-request | Typed validation, idempotency, request lifecycle, safe retry |
| Malicious user process | Invokes public D-Bus APIs, forges labels, races parameters, creates lookalike windows, spams prompts, reads user files | Kernel-derived identity, trusted prompt, throttling, exact binding, least privilege |
| Malicious standard user | Has a local session but no owner authority | Owner authentication for owner-only tiers; active-session and account policy checks |
| Compromised app with portal grants | Can access only granted resources and its own data | Do not turn resource access into system authority; bind to app identity |
| Remote helper | May view/control a consented session | Visible session, separated capabilities, local approval policy, immediate revocation |
| Social engineering content | Tells the person to approve an unexpected change | Origin and consequence clarity, user-initiation proof, low prompt volume, safe denial |
| Partial system failure | Network loss, power loss, disk full, helper crash, update failure | Transactionality, snapshots, durable state machine, rollback and recovery |
| Supply-chain compromise outside LNP | Signed source or upstream package is malicious | Source transparency, sandbox preference, signature enforcement, containment; no claim of perfect prevention |

#### 13.2 Threats specifically addressed by trusted intent

- A background process silently performing a Tier 2 system change.
- A requesting process injecting input into the real trusted surface.
- Another client overdrawing, obscuring, repositioning, or imitating the trusted
  indicator while the real decision is active.
- Unconsented screen capture of sensitive authentication UI.
- Prompt swapping: presenting operation A and executing B.
- Replay of a prior approval or response.
- Approval racing with parameter or requester replacement.

#### 13.3 Out of scope or only partially mitigated

- A person who knowingly approves the accurately described harmful action.
- Malware reading or destroying files already available to the compromised
  account. Portals and app sandboxing reduce this; elevation UI cannot solve it.
- A kernel, polkit authority, intent broker, privileged mechanism, or trusted
  compositor compromise.
- Physical attacks after disk unlock, firmware attacks, malicious peripherals,
  and hardware keyloggers.
- Denial of service by killing session components or flooding resources. Prompt
  throttling limits one form, but availability is not guaranteed against a
  compromised account.
- A visually perfect fake shown while no real decision is active. Trusted
  indicators and user education can make the real path distinguishable, but
  cannot prevent an untrusted app from drawing a picture. Crucially, typing a
  reusable password into a fake is more harmful than clicking it; LNP should
  migrate common authentication toward protected system UI and hardware-backed
  methods.

#### 13.4 Trust boundaries

| Component | Trust | Responsibilities | Must not do |
| --- | --- | --- | --- |
| Requesting app | Untrusted | State desired job; provide typed parameters and parent/activation context | Supply trusted copy, credentials, policy, or privileged commands |
| Session bus / portal frontend | Transport and UX routing, not authority | Bind request to caller; manage lifecycle; locate active session | Treat caller token as authorization |
| System intent broker | Trusted, minimal | Resolve policy, verify requester/session, normalize operation, select tier, coordinate decision, audit | Parse arbitrary shell/text or perform broad operation itself |
| Authentication authority (polkit/PAM today) | Trusted | Authenticate allowed identity and return policy result | Expose credential to requester or claim consent was authentication |
| Trusted presenter (future KWin path) | Security-sensitive trusted computing base | Render immutable system decision, take genuine local input, enforce accessibility, report one result | Accept caller-authored meaning or return reusable authority to caller |
| Privileged mechanism | Trusted, smallest possible | Revalidate normalized parameters, perform one operation transactionally, return structured result/undo | Trust GUI validation, inherit caller environment, or broaden scope |
| Audit/undo store | Trusted integrity, privacy-sensitive | Durable lifecycle, result, and recovery record | Store secrets or become a bearer capability |

The future trusted presenter makes KWin part of the security boundary it already
approaches in practice. Before claiming that boundary, upstream design must
address same-UID tracing, user-loaded compositor code, registration spoofing,
screen readers, magnifiers, remote assistance, screen capture, session lock,
multi-seat, nested compositors, and compositor restart. LNP must not ship a
“secure-looking” private protocol and declare those solved.

### 14. Operation risk ladder

Risk is assigned to the *normalized operation*, not to the app, helper, or
marketing category. Two buttons on one page may require different treatment.

| Tier | Meaning | Human interaction | Examples | Required properties |
| --- | --- | --- | --- | --- |
| 0 — Automatic | Safe, deterministic, policy-bounded, and either nonpersistent or automatically recoverable; safe even if invoked by a hostile user process | None, or ordinary non-security UI | Read status; apply an unset user default with backup; clear a strictly bounded cache; retry a service; scheduled signed update under an already approved policy and successful snapshot | Idempotent; strict limits; no new trust source; useful audit; invocation cannot meaningfully harm confidentiality, integrity, or availability |
| 1 — Resource grant | Gives one app access to a user-owned resource or session capability | Just-in-time portal chooser/permission | Open selected file; camera; microphone; screen sharing; remote desktop; USB device | App identity; exact resource; visible active state where relevant; revoke/expiry; no root authority |
| 2 — Trusted intent | Bounded, understandable, reversible system change that an owner may approve by protected, explicit intent | Compositor-owned trusted decision; until available, use authentication or do not expose | Restore the expected label on one identified file; enable an allowlisted reversible security setting; install signed updates from already trusted sources after snapshot | Owner eligibility; exact scope; root-owned copy; independent validation; undo; one-shot; rate limit; active local session |
| 3 — Local authentication | Persistent, broad, source-changing, executable-code, account, security-boundary, boot, or difficult-to-reverse change | Trusted local authentication for every exact operation | Add RPM Fusion and codecs; proprietary driver; new repository; persistent file-context rule; user/account changes; system restore | Tier 2 properties plus approved identity, no retained authority, stronger confirmation for unusual scope, recovery plan |
| 4 — Expert/support boundary | Arbitrary or poorly bounded exception whose consequences LNP cannot accurately summarize or contain | No normal one-click path; authenticated, deliberately entered expert/support workflow | Generate/install policy from arbitrary SELinux denials; run arbitrary command as root; disable SELinux/firewall; import unsigned key or package; edit boot policy | Preserve diagnostics; explain why no safe automatic fix exists; require explicit expert context; log and offer recovery where possible |

Tier 0 is not “low risk enough.” It means safe under hostile invocation within
the declared system model. If repeatedly starting an otherwise harmless task
can fill the disk, drain the battery, disrupt the network, or erase the last
recovery point, it is not Tier 0 until quotas and serialization remove that
harm.

Tier 2 is intentionally small. Reversibility alone is insufficient: a reversible
firewall change can still end a remote session; a reversible software install
can execute malicious code before undo. The person must be able to understand
the exact object and consequence in one short decision.

#### 14.1 Account role policy

- An eligible owner in the active local session may approve Tier 2 with trusted
  intent when that facility exists.
- A standard user must authenticate an eligible owner for Tier 2 and Tier 3.
- An owner must authenticate for Tier 3. Merely belonging to `wheel` is not
  recent proof of identity.
- Locked, inactive, remote-only, and background sessions cannot create trusted
  intent. Policy for remote authentication must be explicit and separate.
- Recovery mode uses a dedicated policy described in §14.3; it must not silently
  inherit the normal session assumption.

#### 14.2 Classification of current LNP operations

| Current operation | Target tier | Design decision |
| --- | --- | --- |
| Read system state in Welcome/Computer Care | 0 | Keep read-only and silent; handle unavailable data as unknown, not false |
| Enable/disable the user-owned dock | 0 | No authorization; preserve state and report failure inline |
| Apply or restore the person's Plasma layout | 0 with ordinary confirmation for restore | User-owned and backed up; do not use a security prompt |
| Vacuum bounded old logs and package downloads | 0 only after hard size/age floors | Split from broad cleanup; safe under hostile repeat invocation |
| Remove unused Flatpak runtimes | 0 for user-owned runtimes; separately bounded for system runtimes | Preview reclaimed size; never remove an installed app |
| Prune system snapshots | 2 | Preserve newest known-good, newest pre-update, and minimum count; show what safety history is removed |
| Install Fedora-signed updates from already enabled trusted sources | 2 when user-triggered; 0 under a separately approved automatic-update policy | Require successful snapshot and transaction validation; source set may not change |
| Add RPM Fusion and media packages | 3 | New trust sources and executable code; one exact authentication, clear source/consequence, snapshot |
| Install NVIDIA's driver through RPM Fusion | 3 | Executable kernel code and boot/graphics risk; snapshot, Secure Boot handling, build completion and reboot guard |
| Install `setroubleshoot-server` from an existing Fedora source | 2 | Exact signed package, no source change, undo available |
| Enable firewall locally | 2 if no remote-control/SSH dependency; otherwise 3 or defer | Preview affected remote services; create undo timeout for remote contexts |
| Enable firmware refresh timer | 2 | This authorizes checking, not installing arbitrary firmware; actual update follows fwupd policy |
| Set opportunistic DNS-over-TLS | 2 | Separate operation; connectivity probe and timed rollback required; do not hide in “security checkup” |
| Restore the expected SELinux label on one exact inode/path | 2 | Broker resolves path safely, shows object, expected/current label, and rejects path replacement |
| Recursive relabel of a bounded approved tree | 3 | Broad scope and runtime side effects; preview count/scope, authenticate, log undo limits |
| Toggle an allowlisted SELinux boolean | 2 or 3 per root-owned allowlist | Metadata must explain concrete consequence and prior state; unknown booleans are Tier 4 |
| Add a persistent SELinux file-context rule | 3 | Persistent security policy; exact normalized path expression and type, duplicate-aware undo |
| Build an allow module from recent denials | 4 | Remove from ordinary UI. “Recent” is not the selected incident and `audit2allow` cannot supply product-level intent |
| Restore a pre-update system snapshot | 3 in normal UI; recovery policy at failed login | Show snapshot date/reason and that apps/system changes after it will be rolled back; preserve replaced root |
| Open a helper shell | 4 | Helper-only, explicitly labelled, never the recommended recovery action |

#### 14.3 Recovery authorization policy

A broken graphical login is precisely when normal trusted UI and convenience
credentials may be unavailable. Recovery must balance unauthorized rollback
against preventing a normal owner from regaining a usable computer.

The supported recovery flow is:

1. The boot or display failure handler presents an accessible recovery surface
   on the local console, with locale, keyboard layout, magnification, and screen
   reader support available before authentication.
2. “Undo the last system update” identifies the exact saved state and describes
   what will and will not change.
3. When the disk is already unlocked by the same local boot and the recovery
   target is an LNP-created, integrity-verified pre-update snapshot, owner
   authentication is preferred. A recovery key path must exist if PAM is
   broken by the update.
4. A distribution may explicitly choose physical-presence recovery without a
   reusable credential only after documenting the local attacker model. That
   policy must be visible during setup, cannot expose home data, and may restore
   only a system-generated target—not arbitrary root state.
5. The replaced root is retained and the result is auditable. The next boot
   clearly says that a recovery occurred and offers “Keep this restored system”
   or “Get help”; it does not silently delete either state.

The current text-console menu is a useful last-resort prototype, not the final
accessible recovery experience. Its automatic retry may remain, but it needs a
pause option for people who require more time.

### 15. Architecture: present-safe path

LNP can improve substantially without forking polkit or claiming a new security
boundary. This is the required architecture before trusted intent exists.

#### 15.1 Replace broad `pkexec` helpers with typed mechanisms

The preferred mechanism is a small system D-Bus service with one method per
operation family and a separate polkit action ID at each policy boundary. The
service obtains caller credentials from D-Bus, validates inputs, calls polkit
with the exact action ID and structured details, then performs only that
operation. A set of fixed-purpose executables is acceptable as an interim if
each executable has its own policy and closed arguments. A generic root helper
whose subcommand selects codecs, drivers, cleanup, and security is not.

Minimum action split for current code:

```text
org.lnp.software.add-media-support          Tier 3 / auth_admin
org.lnp.drivers.install-nvidia              Tier 3 / auth_admin
org.lnp.maintenance.clear-system-cache      Tier 0 / yes only after hard bounds
org.lnp.maintenance.prune-snapshots         Tier 2 / auth_admin until trusted intent
org.lnp.security.install-explainer          Tier 2 / auth_admin until trusted intent
org.lnp.security.enable-firewall            Tier 2 or 3 after context evaluation
org.lnp.security.enable-firmware-checks     Tier 2 / auth_admin until trusted intent
org.lnp.network.enable-opportunistic-dot     Tier 2 / auth_admin until trusted intent
org.lnp.selinux.relabel-one                 Tier 2 / auth_admin until trusted intent
org.lnp.selinux.relabel-tree                Tier 3 / auth_admin
org.lnp.selinux.set-allowlisted-boolean     Per-entry Tier 2/3
org.lnp.selinux.add-file-context            Tier 3 / auth_admin
org.lnp.recovery.restore-snapshot           Tier 3 / auth_admin or recovery policy
```

There must be no `_keep` result. The polkit manual warns that retained results
for the same action and subject can apply when passed details differ. More
fundamentally, retained authority teaches the wrong mental model: one password
does not mean “the rest of this wizard may change anything.”

#### 15.2 Present-safe interaction

Until the trusted presenter passes upstream security and accessibility review:

- Tier 0 runs without a security prompt only after hostile-invocation review.
- Tier 1 uses existing portals.
- Tier 2 uses the ordinary system authentication path, even for an owner. This
  is more friction than the north star, but it is honest.
- Tier 3 uses `auth_admin`, one operation per authentication.
- Tier 4 has no normal action button.

LNP must not ship a custom polkit agent that returns authentication success
after a Yes click. `AuthenticationAgentResponse2` means an identity was
authenticated. Reusing it for unauthenticated consent makes the agent and
policy audit trail lie. Empty identity lists and an informal `consent-only`
detail are not a stable protocol or a safe old-agent fallback.

The app should use `pkexec --disable-internal-agent` only when a known graphical
agent is present and report “Owner approval isn't available in this session”
with retry/help options otherwise. It must never fall back to embedding a
terminal password prompt.

This interim is “present-safe” only in the sense that it narrows authority and
uses existing APIs honestly. An ordinary session polkit window is not promoted
here to an unspoofable boundary: same-account malware may still draw a password
lookalike or attack session components. Tier 3 therefore inherits the supported
platform's current authentication-agent risks until the trusted path is
reviewed and deployed.

#### 15.3 Mechanism rules

Every privileged mechanism must:

- start from a minimal environment and absolute executable paths;
- use fixed argv vectors, never a shell command line, `eval`, or caller-provided
  executable;
- normalize and validate identifiers, paths, URLs, package/source identities,
  and enum values independently of the GUI;
- resolve file operations with descriptor-relative APIs and protections against
  symlink/path replacement races; string checks for `..` or shell characters
  are not sufficient filesystem authorization;
- set resource, duration, output, and concurrency limits;
- create a snapshot or operation-specific undo state before change when
  promised;
- expose structured progress and error codes, keeping human copy in the UI;
- be idempotent or detect an already completed operation safely;
- survive client disconnect without broadening or repeating work; and
- emit a durable, privacy-reviewed audit and undo record.

Package/repository operations additionally pin the expected repository
definition, HTTPS origin, signing-key fingerprint, supported Fedora release,
package allowlist, and transaction preview. The GUI cannot pass a release-RPM
URL to root and call that bounded. Root-owned metadata determines the source.

### 16. Architecture: future trusted intent authorization

The long-term design is an upstream **intent authorization service**, not a
private cosmetic UAC clone and not an in-tree set of permanent polkit/KWin
forks. Names below are conceptual until accepted upstream.

#### 16.1 Components

```text
untrusted app
    │ Begin(action ID, typed parameters, parent, activation context)
    ▼
system action broker / privileged mechanism
    │ normalize + validate + resolve root-owned action manifest
    │ request decision bound to caller/session/operation digest
    ▼
intent authority (system service)
    │ policy: automatic / resource portal / intent / authenticate / refuse
    ├──────── Tier 3 ───────► system authentication authority (PAM/fprint/FIDO)
    │
    ▼
registered trusted presenter for the active local session
    │ compositor-owned semantic UI; exclusive genuine decision input
    ▼
human
    │ one response bound to request digest
    ▼
intent authority ──► broker executes the already-normalized operation
```

The caller never receives a reusable elevation token. The broker retains the
operation and executes it after the authority returns a positive decision. If
a separate mechanism must execute, the authority addresses a one-use decision
directly to that mechanism over an authenticated channel; the app cannot spend
or modify it.

#### 16.2 Root-owned action manifest

Each operation has installed, signed/root-owned metadata. An illustrative
schema is:

```json
{
  "id": "org.lnp.selinux.relabel-one",
  "version": 1,
  "risk_tier": 2,
  "eligible_roles": ["owner"],
  "mechanism": "org.lnp.SystemActions1.RelabelOne",
  "parameters": {
    "file_handle": {"type": "local-file-handle", "required": true}
  },
  "presentation": {
    "title_id": "relabel-one-title",
    "consequence_id": "relabel-one-consequence",
    "primary_verb_id": "repair-access",
    "object_renderer": "local-file-display-name",
    "shows_requester": true,
    "reversible": true
  },
  "limits": {"max_objects": 1, "timeout_seconds": 60},
  "undo": "org.lnp.selinux.restore-label"
}
```

Localized strings are installed system resources keyed by IDs. Parameters are
rendered by type-specific trusted renderers. A client string is displayed only
as explicitly quoted untrusted data, length-limited and never interpolated as
markup. The authority rejects an unknown version, unknown field, missing
renderer, or mismatch between manifest and mechanism.

#### 16.3 Request identity and digest

The broker derives and freezes:

- system-bus unique name and kernel UID/PID;
- process start time and executable identity;
- sandbox/app identity when verifiable;
- logind session, seat, local/remote flag, active and locked state;
- action manifest version;
- canonical typed parameter encoding;
- resolved object identity (for files, an open descriptor plus device/inode,
  not merely a pathname);
- monotonic creation/expiry time; and
- a cryptographic operation digest over the above.

The display name shown for the requester comes from a trusted desktop entry or
sandbox metadata matched to the verified executable/app identity. Otherwise it
says “An unidentified app” and Tier 2 cannot proceed. The caller cannot improve
its trust merely by choosing a familiar name or icon.

An activation token or recent input serial helps establish that the request
followed a real user action in the claimed window. It reduces background prompt
spam and improves origin association, but does not replace policy or approval.

#### 16.4 Request lifecycle and API properties

The API should follow the portal request pattern:

```text
RECEIVED → VALIDATING → AWAITING_DECISION → AUTHORIZED → EXECUTING → SUCCEEDED
       └──────────────► REJECTED
                         ├───────────────► DENIED
                         ├───────────────► CANCELLED
                         ├───────────────► EXPIRED
                         └───────────────► SUPERSEDED
AUTHORIZED / EXECUTING ─► FAILED (with unchanged/rolled-back/partial state)
```

- `Begin` returns a caller-bound request object immediately.
- The caller subscribes before the call using an unguessable handle token to
  avoid a response race, as portals do.
- The request exposes read-only structured phase/progress suitable for the
  requesting UI.
- `Close` cancels only before the commit point. After commit it detaches the UI;
  it does not pretend the system operation stopped.
- One structured terminal response is emitted. The request then becomes inert.
- Request object name and token provide correlation, not authority.
- A client disconnect cancels an uncommitted request and cannot cancel a
  committed transaction whose interruption would be unsafe.
- The broker serializes conflicting operations and reports the existing task
  rather than opening a second prompt.

#### 16.5 Trusted presenter requirements

The presenter must be owned by the compositor or another upstream-reviewed
trusted path with equivalent guarantees. While a decision is active it must:

- be unmistakably system-owned without relying on a secret visual the person
  must memorize;
- render only authority-supplied structured content and trusted localizations;
- remain above untrusted surfaces and resist overdraw, repositioning, input
  redirection, focus theft, and clipboard substitution;
- accept only genuine input delivered by the compositor or a separately
  authorized accessibility path;
- block unapproved synthetic input, automation, and remote-control input;
- prevent untrusted screen capture of credentials and define explicit policy
  for capture of nonsecret intent prompts;
- cancel on session lock, user switch, seat change, presenter restart, authority
  disconnect, or operation digest mismatch;
- expose role, name, description, object, state, risk, focus order, buttons, and
  status through a protected semantic accessibility interface;
- honor font, locale, direction, contrast, reduced motion, screen reader,
  magnifier, switch, and keyboard needs from the first frame; and
- allow a person to inspect requester and technical provenance without leaving
  the trusted context.

A `.desktop` key such as `X-KDE-Wayland-Interfaces` is not sufficient evidence
that a client is trusted: user processes can launch, stop, trace, or imitate
session components unless the complete lifecycle is hardened. Registration
needs kernel-derived peer identity, root-owned executable and deployment
verification, one presenter per active session, replay-resistant handshake,
and protections against same-UID tracing/injection. User-provided KWin scripts,
effects, themes, or plugins must be shown not to modify the trusted path.

These are upstream design questions, not details LNP may wave away. Until the
review has answers, the feature remains experimental and LNP uses Tier 3
authentication for would-be Tier 2 actions.

#### 16.6 Distinct protocol results

If implemented in polkit, consent must be a first-class policy and wire result:

- a named policy outcome such as `intent_required`, with **no retained form**;
- a distinct `BeginIntentAuthorization` request, not an empty identity list
  smuggled through `BeginAuthentication`;
- a distinct response meaning “the protected presenter reports this operation
  digest was approved,” not `AuthenticationAgentResponse2`;
- version/capability negotiation where old authorities and agents fail closed;
- explicit authority verification of the registered trusted presenter; and
- audit terminology that never says an identity authenticated when it did not.

It may be cleaner to place this alongside portals and let polkit remain an
identity-authentication authority. That choice belongs in an upstream threat
model with polkit, KDE, GNOME, portal, accessibility, and distribution
maintainers. LNP's product requirement is the semantic distinction and security
properties, not ownership by a specific project.

#### 16.7 Authentication path

Tier 3 should use the same trusted system presentation framework but ask the
allowed identity to authenticate. The requesting app never hosts or reads the
credential. The authority chooses available methods—password, fingerprint,
security key, or future hardware-backed presence—and returns only a decision
bound to the operation.

Requirements:

- show which owner identity is requested and permit switching among policy-
  allowed identities;
- never imply a fingerprint alone explains or consents to the displayed
  operation; consequence and identity must both be present;
- offer an accessible alternative when biometric use is impossible;
- rate-limit failed authentication without enabling permanent local lockout;
- clear credential buffers and prevent them from entering logs, crash reports,
  clipboard, accessibility speech history, or screenshots;
- do not cache authority across operations; and
- record authentication method class, not secret material.

#### 16.8 Decision presentation

The normal Tier 2/3 prompt has this semantic order:

1. **Action:** “Install NVIDIA's graphics driver?”
2. **Requester:** “Requested by Computer Care.”
3. **Consequence:** “This adds software from RPM Fusion, changes the graphics
   driver used at startup, and needs a restart.”
4. **Scope/provenance:** a short trusted row such as “NVIDIA driver • this
   computer • RPM Fusion.”
5. **Recovery:** “A saved system state will be created first.”
6. **Choice:** `Cancel` and the exact verb, `Install driver`.
7. **Details:** installed packages, source/key, request ID, and support link.

The safer/cancel action is available immediately and receives initial keyboard
focus unless platform accessibility evidence supports another choice. An
unusual Tier 3 action may use a brief inhibitive attractor—for example, keeping
the consequential button disabled until the operation/scope summary has been
focused or exposed for a minimum reading opportunity—but delays must not become
ritual. Research found inhibitive attractors made informed security decisions
two to three times more likely in tested dialogs
([Bravo-Lillo et al.](https://www.microsoft.com/en-us/research/publication/your-attention-please-designing-security-decision-uis-to-make-genuine-risks-harder-to-ignore/)); LNP must test its own implementation with disabled and low-literacy users.

No prompt may say only “Authentication is required,” “Allow changes?”, or
“needs administrator privileges.” No positive button may say `Yes`, `OK`,
`Continue`, or `Allow` without the action. Prompts must not contain a countdown
to approval. Timeout always denies.

#### 16.9 Prompt-spam and habituation defense

Permission UI fails when people stop attending to it. In a study of Android's
install-time permissions, only 17% paid attention and 3% answered all tested
comprehension questions correctly
([Felt et al.](https://research.google/pubs/android-permissions-user-attention-comprehension-and-behavior/)). Repeated non-security dialogs can also reduce attention and adherence to a
novel security warning through generalized habituation
([Vance et al.](https://www.usenix.org/conference/soups2019/presentation/vance)).

The broker therefore enforces:

- a recent user-activation context for app-initiated Tier 2/3 prompts;
- at most one visible decision per seat and one pending request per caller/action;
- duplicate coalescing by operation digest;
- exponential cooldown after denial, cancellation, or timeout;
- a session and daily ceiling after which requests are logged and shown as
  blocked in Computer Care rather than interrupting;
- immediate cancellation when the requester exits or changes identity;
- no prompt from an inactive or locked session; and
- a visible “This app has asked repeatedly” state with `Keep blocked` as the
  safe action.

System-initiated maintenance cannot open a surprise Tier 2/3 prompt. It posts a
quiet, durable item that the person opens, converting it into a user-initiated
request.

#### 16.10 Accessibility and trusted input

Assistive technology is part of the threat model, not an afterthought. Reading
the semantic tree can expose a password; invoking an accessible button can be
equivalent to synthetic input; magnification and screen sharing may capture the
surface. Blanket denial would exclude people from administering their own
computer.

The upstream design must specify separate capabilities for:

- semantic read access to nonsecret prompt content;
- protected entry of secret text without speech/log history;
- focus navigation and activation from switch/speech/keyboard systems;
- compositor magnification and contrast transformation;
- local assistive input versus remote-control input; and
- time-bounded setup of a trusted accessibility service.

Any trust grant to an assistive service must be installed through root-owned
metadata, bound to verified code and a specific capability, visible and
revocable, and established with local owner authentication. A generic same-UID
AT-SPI client cannot be allowed to auto-approve trusted prompts. Windows'
`UIAccess` model illustrates the tension between automation/accessibility and
protected UI, but LNP needs a Linux-native upstream design rather than a broad
allowlist.

#### 16.11 Remote assistance policy

By default, Tier 2 and Tier 3 decisions require local physical input. A remote
desktop session may display that a decision is waiting but cannot activate it
or enter an owner credential. This stops an app from turning screen-control
permission into administrator authority.

An accessibility user who depends on remote input needs an explicit supported
mode, not an undocumented bypass. Designing that mode requires separate local
setup/authentication, a named trusted helper or hardware-backed remote identity,
per-capability grants, conspicuous active status, short expiry, and an emergency
local revocation route. It is a release blocker for claiming remote-accessible
administration, not a reason to weaken every local prompt.

#### 16.12 Audit, undo, and transparency

Each request record contains:

```text
request UUID; action ID + manifest version; normalized scope summary;
verified requester identity; account/session/seat; created/decided/completed time;
required tier; decision result; authentication method class if applicable;
mechanism version; operation result; rollback/undo ID; redaction version
```

Computer Care presents this as human history: “Media support was installed from
RPM Fusion today at 2:14 PM,” with source, result, and available undo. Technical
export contains exact package NEVRAs and journal correlation IDs. Sensitive
paths use a stable per-record display alias or redacted parent unless the person
chooses to reveal them.

The audit store is root-owned, append-resistant, size-bounded, rotated, and
covered by tests for clock changes and disk-full behavior. It is not proof that
the operation was wise; it is evidence of lifecycle and scope.

Undo is an operation with its own classification. Possessing an undo ID does
not authorize it. The mechanism checks current state and refuses a destructive
“undo” when later dependent changes make it unsafe, explaining the alternative.

### 17. Security verification plan

The authorization implementation needs independent threat-model review and the
following evidence before stable use:

| Area | Required tests |
| --- | --- |
| Caller identity | Spoofed app name/icon/PID; D-Bus name transfer; process exit/restart; sandbox identity mismatch; inactive/remote session |
| Parameter binding | Mutation after display; Unicode confusables; overlong strings; unknown fields; canonicalization collisions; path rename/symlink/mount replacement; file deletion/recreation |
| Replay/race | Reused response, request token, digest, undo ID; simultaneous approve/cancel; double click; authority/presenter restart; clock rollback |
| Mechanism | Invalid enum/identifier/path/package/source; hostile environment; output flood; timeout; concurrent invocation; disk full; network loss; SIGKILL/power loss at every commit boundary |
| Presenter | Fake lookalike before/during prompt; overdraw; focus theft; fake input; screen capture; clipboard; nested compositor; KWin script/effect; same-UID ptrace; lock/user switch/multi-seat |
| Authentication | Wrong identity; repeated failure; cancel; biometric unavailable; password paste; secret in logs/crash/core/accessibility history |
| Spam | Burst callers, duplicate actions, requester churn, system-initiated surprise, ceiling reset, denial cooldown |
| Accessibility | Orca read and activate; keyboard-only; switch/speech path if supported; magnifier; high contrast; 200–400% text; reduced motion; slow input; timeout extension without auto-approval |
| Audit/undo | Redaction, log tamper attempt, rotation/disk full, incomplete transaction, undo after later change, recovery after reboot |

Fuzz typed request decoders and manifest parsing. Property-test operation
canonicalization so semantically different requests cannot share a digest and
equivalent normalized requests do. Run privileged integration tests in a VM
with snapshots and fault injection. The reviewer who authored the mechanism
must not be the only security approver.

### 18. Explicitly rejected authorization designs

- **A custom agent that clicks through `auth_admin`:** it falsely reports
  authentication and turns agent trust into an undocumented bypass.
- **`auth_admin_keep` for a wizard/session:** details or later subcommands can
  differ; one credential must not fund unrelated work.
- **`auth_consent_keep`:** repeated ambient authority is the opposite of exact
  intent.
- **Ordinary app-authored confirmation before `pkexec`:** useful for preventing
  accidental clicks, but untrusted against a malicious requester and not a
  replacement for system authorization.
- **Caller-supplied prompt text or icon:** allows semantic spoofing even if the
  window itself is protected.
- **A cookie as a capability:** request handles correlate; possession must not
  grant root work.
- **A Wayland protocol gated only by desktop-file metadata:** does not prove the
  running process, lifecycle, code, or same-UID isolation.
- **One broad privileged helper with a “validated” subcommand:** broadens every
  policy mistake and makes action-specific authorization/audit ambiguous.
- **Security by confirmation volume:** warnings become habituation training.
- **An expert confirmation for generated SELinux allow rules:** a normal person
  cannot inspect the policy consequence, so the system cannot obtain informed
  approval through better copy.

---

## Part III — Core-journey research and test matrix

### 19. Purpose and research questions

The research program finds what the team does not know. It is not a demo, a
preference survey, or a final exam for participants. The primary questions are:

1. Can people transfer existing Windows/macOS knowledge to LNP's shell without
   Linux-specific instruction?
2. Where do labels, layout, and system ownership violate their mental model?
3. Which common hardware/software tasks still lead to a terminal, web search,
   or external helper?
4. Do people notice, understand, and safely act on authorization requests,
   including an unexpected or suspicious request?
5. Can people recover from partial work, update failure, low disk, and failed
   graphical login without risking personal files?
6. Are notifications actionable and proportionate, or do they train dismissal?
7. Can disabled people complete the same journeys independently with their own
   strategies and assistive technologies?
8. Can a helper diagnose and assist without receiving standing access, hidden
   privilege, or unnecessary personal data?
9. Does performance hold after the novelty of onboarding and after a gap in use?

Every product assumption is written as a falsifiable question or hypothesis.
“People will understand the dock” becomes “At least seven of eight shell-naive
participants can launch, switch, minimize, and recover a hidden window without
instruction; failures reveal which state cues or labels are missing.”

### 20. Research cadence and methods

Research is continuous. GOV.UK's service guidance recommends small batches in
every iteration, typically four to eight participants per usability round, and
including disabled users and people needing support throughout
([planning research](https://www.gov.uk/service-manual/user-research/plan-user-research-for-your-service),
[participant recruitment](https://www.gov.uk/service-manual/user-research/find-user-research-participants)).

LNP uses four complementary tracks:

| Track | When | Sample | Purpose |
| --- | --- | --- | --- |
| Discovery/context | Before or alongside design | 6–12 visits/interviews across profiles; contextual use where possible | Understand real devices, support relationships, vocabulary, interruptions, and workarounds |
| Iterative formative test | At least once per two-week design iteration while a critical journey changes | 4–8 people, deliberately varied; include access needs every round where practical | Find and fix severe failures quickly; compare revised flows |
| Accessibility specialist round | Before beta and after relevant platform change | At least one experienced user for each applicable modality plus expert audit | Validate real assistive-technology strategies and trusted/recovery paths |
| Summative benchmark | Stable-release candidate | Pre-registered sample sized for journey targets; initially plan 60+ across key strata and revise with a statistician | Estimate completion, safety, time, comprehension, and satisfaction with confidence intervals |

Expert heuristic review, automated accessibility inspection, code review, and
dogfooding are useful but never replace representative observation. Preference
questions are not evidence that a task works. Production support reports reveal
issues but systematically omit people who abandon or never install.

### 21. Recruitment

#### 21.1 Inclusion matrix

Across each three-round cycle, recruitment must cover:

- no prior Linux use (majority), plus a small comparison group with incidental
  Linux exposure;
- recent primary use of both Windows and macOS;
- low and moderate digital confidence, including people who normally ask
  someone else for setup/help;
- a spread of ages, literacy/reading confidence, education, income, and primary
  language; translated builds when localization is in scope;
- laptop and desktop use; touchpad, mouse, touchscreen, and multi-monitor
  contexts;
- owner and household standard-user roles;
- people who use screen readers, magnification, high contrast, keyboard-only,
  speech input, switch/alternative input, reduced motion, or hearing supports;
- people with cognitive, learning, attention, fatigue, or memory-related access
  needs; and
- trusted helpers who routinely support another person's computer.

Do not recruit only technology enthusiasts or people comfortable installing an
experimental OS. Provide a managed test device or accessible remote lab so
participation does not require installing LNP.

For a formative round of eight, a reasonable starting mix is six people with
no Linux use, at least four who describe low/moderate confidence, two aged 65+,
two who use a relevant assistive technology, two household standard-user
scenarios, and a mix of Windows/macOS backgrounds. These attributes may overlap
and are adjusted to the round's research question.

#### 21.2 Ethics, consent, and compensation

- Provide a plain-language information sheet in the participant's preferred
  accessible format.
- Say explicitly that the software is being tested, not the participant, and
  that they may stop, skip, or withdraw without losing compensation.
- Obtain separate choices for observation, audio, video/screen recording,
  keystroke/event logging, quotes, and future contact.
- Never ask for a real password, personal account, real files, biometric
  enrollment, or live remote-control grant. Use study identities and fixtures.
- Avoid collecting disability diagnoses when functional access needs suffice.
- Compensate time, preparation, travel, personal-assistant/interpreter time, and
  reasonable access costs promptly.
- Redact names and incidental personal information. Set a retention date and
  delete raw recordings on schedule.
- Do not publish a participant's unsafe decision as ridicule. Report the design
  conditions that made it reasonable or habitual.

### 22. Test environment and fixtures

Every tested build is identified by Git commit, package versions, Fedora/KDE
versions, locale, display scale, assistive technology, firmware/VM image, and
test-data seed. A reset script or VM snapshot establishes known starting state
before each session.

The test lab needs:

- one supported Intel/AMD laptop with touchpad and integrated graphics;
- one supported NVIDIA machine with Secure Boot permutations;
- Bluetooth headset and keyboard, common USB printer/scanner, removable drive,
  camera/microphone, and external display;
- controllable Wi-Fi with correct, wrong-password, captive-portal, slow, and
  disconnect states;
- package mirror/source fixtures for success, timeout, signature failure,
  insufficient space, and partial download;
- Btrfs images with known pre-update snapshots and separate fixture home data;
- owner and standard accounts with non-personal study credentials;
- an instrumented malicious-prompt simulator that never receives a real secret
  or privilege;
- Orca, KWin magnifier, high-contrast themes, 200% and 400% scaling, on-screen
  keyboard, and supported alternative-input tools; and
- local and remote helper fixtures with visible capability/revocation state.

The prototype must work end-to-end. A facilitator may manually drive a clearly
documented “Wizard of Oz” backend only when the research question concerns
interaction rather than trust, latency, error, or accessibility. Never use a
fake prompt to claim the security architecture works.

### 23. Session protocol

#### 23.1 Opening

The facilitator says, in substance:

> We are testing this computer, not you. Some parts may be unfinished. Please
> work as you normally would; you cannot break the study machine. I may wait
> quietly when something is confusing because that shows us what to improve.
> You can stop or skip anything. Please do not enter a real password or use a
> personal account.

Collect background and access setup before tasks. Let participants configure
their own assistive technology where that is their normal practice; observe
whether first-run discovery is itself the research question.

#### 23.2 Task wording

Tasks state a human goal and context, not the control or feature to use. Good:
“A friend sent you this video. Make it play.” Bad: “Open Computer Care and
install the codecs.” Good: “The screen is hard to read today. Make everything
bigger.” Bad: “Set display scaling to 200%.”

Avoid leading words copied from button labels. Provide realistic fixture data.
Do not reveal that a security request is malicious before the participant
decides.

#### 23.3 Assistance ladder

The facilitator records, in order:

0. no assistance;
1. neutral prompt: “What are you thinking?”;
2. restate the goal without new nouns;
3. broad location cue: “Where might the computer keep settings?”;
4. specific control instruction;
5. facilitator rescue/completion.

Only level 0 is independent completion. Levels 1–3 remain useful evidence but
are coded as assisted. Safety-critical scenarios use retrospective probing
after the decision so asking “what are you thinking?” does not artificially
increase attention.

#### 23.4 Retrospective questions

After each consequential prompt, remove or mask it and ask without offering
answers:

- What was asking?
- What did it want to do?
- What part of the computer would change?
- Had anything changed yet?
- Could the change be undone? How?
- Why did you choose that button?
- What would you expect to happen next?

Then show the prompt again and ask what, if anything, was unclear. This
separates unaided comprehension from recognition after re-reading.

After a task, record a 1–7 Single Ease Question and ask one concrete follow-up:
“What was the hardest or least certain moment?” At session end, ask for overall
confidence and expected support needs, not merely preference between visual
styles.

### 24. Observation and scoring

#### 24.1 Outcome codes

| Code | Meaning |
| --- | --- |
| SI — safe independent | Goal completed without assistance, terminal, unsafe approval, or violated invariant |
| SA — safe assisted | Goal completed but assistance level 1–4 was needed |
| UF — unsafe completion | Apparent goal completed by accepting excessive scope, disabling a protection, exposing data, or otherwise violating the invariant |
| AB — abandoned | Participant chose to stop or defer because no acceptable route was apparent |
| TF — technical failure | Supported path could not complete due to product/build/hardware failure rather than participant interaction |
| NA — not applicable | Fixture or participant context made the journey irrelevant; never use to hide a failure |

Technical failure is still a product finding. Report both “interaction success
among working fixtures” and end-to-end success; do not discard TF from the
latter.

#### 24.2 Recorded measures

- outcome and assistance level;
- time from task start to stable outcome, excluding agreed study interruption;
- first action, navigation path, backtracks, repeated actions, and dead ends;
- words searched for and controls expected but absent;
- error, notification, prompt, cancel, undo, and help behavior;
- terminal/web/phone/helper attempt;
- authorization comprehension fields and decision confidence;
- accessibility barriers, workarounds, fatigue, and lost focus/context;
- mismatch between expected and actual next state;
- severity and recurrence of each finding; and
- task ease and relevant verbatim comment, within consent limits.

#### 24.3 Finding severity

| Severity | Definition | Release effect |
| --- | --- | --- |
| Critical | Causes unsafe authorization/data exposure, irrecoverable supported-state harm, locks a user out, or makes a critical journey impossible for a supported access modality | Stop release; fix and retest |
| High | Prevents independent completion of a critical journey, leads to terminal/helper dependence, or repeatedly produces materially wrong understanding | Stop stable release; fix and retest |
| Medium | Significant delay, uncertainty, repeated wrong turn, or workaround with eventual safe completion | Must have owner and scheduled fix; fix before stable if recurrent |
| Low | Local clarity, polish, or efficiency issue without meaningful risk or exclusion | Backlog with evidence |

Severity combines impact, reach, frequency, and recoverability. One critical
accessibility or security failure does not become low because only one
participant encountered it.

### 25. Core journey matrix

These are product acceptance journeys, not a permanent script. Each row is
expanded into a study card with exact fixtures and build-specific expected
state. “Independent success” always includes no terminal, copied command, raw
Linux concept, or facilitator help.

| ID | Context and participant task | Fixture/failure branches | Independent success and safety invariant | Required access variants |
| --- | --- | --- | --- | --- |
| J01 First start | “This is your computer now. Get oriented and open the web.” | Clean owner; no pending hardware work; close/reopen onboarding | Finds launcher/browser, understands dock/running state, can locate Settings/Help; onboarding is skippable and does not return as nagware | Keyboard-only, Orca, 200%, low vision, reduced motion |
| J02 Find and resume | Open three apps, minimize/cover one, then: “Go back to the document you were editing.” | Multiple windows for one app; window on another display/activity if supported | Uses dock/overview/switcher; distinguishes running from pinned; no lost-window dead end | Keyboard, screen reader window names, magnifier |
| J03 Adapt display/input | “The text is hard to read and the pointer is difficult to use. Make the computer comfortable.” | Scaling restart/not; high contrast; pointer size; natural scroll preference | Finds relevant settings, preview/undo works, UI remains operable after scale change, choice persists | Low vision, motor, color-vision, 200–400%, RTL locale |
| J04 Wi-Fi | “Connect to the café internet so you can send this document.” | Correct/wrong password, captive portal, disconnect, airplane mode | Finds network state, correct error names next action, retry preserves context, no credential leak | Keyboard/Orca, on-screen keyboard, slow input |
| J05 Bluetooth audio | “Use these headphones for sound and the built-in microphone for the call.” | Pairing timeout, same-name devices, profile/mic selection | Pairs correct device and selects output/input with test/visible state; can forget it | Keyboard/Orca, hearing captions/visual pairing status |
| J06 External display | “Put the presentation on the large screen while keeping your notes on the laptop.” | Hotplug, mirrored default, disconnected/reconnected, scale mismatch | Discovers arrangement/mode, preview timeout safely reverts unusable layout, settings persist by device | Keyboard, low vision/magnifier, no color-only monitor identity |
| J07 Printer/scanner | “Print this form, then scan the signed page back into Documents.” | Auto-discovery, driverless failure, offline device, low paper | Adds/finds device, selects it in app, understands offline state, scans to discoverable location | Keyboard/Orca, magnification, motor target sizes |
| J08 Install/remove app | “Install a trusted drawing app, make a picture, then remove the app without deleting the picture.” | Curated Flatpak, two sources, network loss, disk full | Finds Software, understands app/source when relevant, install survives close, removal distinguishes app from files | Keyboard/Orca, 200%, cognitive/plain-language review |
| J09 Play unfamiliar media | “A friend sent this video. Make it play.” | Missing codecs; offline; repo signature/source failure; owner vs standard user | Reaches contextual media-support path, understands third-party source and owner approval, safely defers/recovers; playback succeeds | Keyboard/Orca, standard-user owner auth, low literacy |
| J10 NVIDIA setup | “Set this computer up so games use its graphics hardware.” | Secure Boot on/off, download failure, akmod build, premature reboot attempt | Understands source/restart, authenticates exact action, sees background build state, unsafe restart is blocked with useful reason, success verified after boot | Keyboard/Orca, slow network, standard user, magnifier |
| J11 Low storage | “The computer says space is running low. Make enough room for the update without losing personal work.” | Large cache, old runtimes, snapshots, personal downloads | Distinguishes safe cleanup from personal files, previews categories, preserves required recovery points, can inspect result; no blanket root authorization | Keyboard/Orca, cognitive, large text |
| J12 Routine update | “Bring the computer up to date and continue working.” | AC/battery, network loss, package conflict, snapshot failure, restart required | Update does not proceed without promised recovery state, UI may close safely, one actionable completion/failure item, restart state clear | Keyboard/Orca, low battery/interruption, slow input |
| J13 Expected authorization | During J09/J10, evaluate the real owner decision | Owner and standard accounts; known requester; exact parameters | Identifies requester/action/scope/reversibility; approves only expected operation; requester never sees credential; cancellation changes nothing | Orca, keyboard, magnifier, alternative auth, long translation |
| J14 Suspicious request | While doing an unrelated task, a fixture app requests a plausible system change | Lookalike app, repeated prompts, misleading caller window; real broker remains safe | Denies or keeps blocked; can identify unexpected origin; no authority after fake prompt; spam is coalesced and task remains usable | Same as J13; test without think-aloud priming |
| J15 Security block | “Photos stopped opening this folder after it was moved. Find out why and fix only this problem.” | One selected mislabeled file plus unrelated recent AVCs; recurrence; file renamed mid-flow | Plain explanation links symptom to exact object, Tier 2/3 decision matches scope, only selected object changes, undo/history available; unrelated denials untouched | Keyboard/Orca, 200%, long path/Unicode, standard user |
| J16 Restore desktop choice | “You preferred the desktop layout you had before LNP. Put it back.” | Valid backup, missing/corrupt backup, live Plasma state | Finds restore without command, understands logout/restart timing, valid restore is exact, failure changes nothing, can reapply LNP later | Keyboard/Orca, magnifier, cognitive |
| J17 Recover failed update | Boot lands in recovery: “Get back to a working desktop and keep your personal files.” | Valid/absent snapshot, failed auth, broken PAM recovery key, restore failure, inactivity timeout | Selects correct dated pre-update state, understands system vs personal files, completes/reverses recovery, reaches desktop; no helper shell required | Screen reader from first frame, keyboard layout, 200%, slow input/no forced timeout |
| J18 Understand a failure | Inject a background service crash that affects printing and unrelated routine churn | Repeated failure, self-repair, old event at login | Only actionable event interrupts; message names impact and realistic next step; duplicate collapse/history/help work | Orca notification reading, hearing visual equivalence, cognitive |
| J19 Get help safely | “You cannot fix this. Ask your trusted helper without sharing unrelated personal information.” | Diagnostic bundle; view-only then control request; helper disconnect | Previews/redacts bundle, grants exact remote capabilities with expiry, sees active state, revokes instantly; privileged prompt follows local policy | Screen reader, keyboard, speech/switch if supported, helper accessibility |
| J20 Household account | “Your child/housemate needs the drawing app but should not be able to change the whole computer.” | Standard account, owner present/absent, wrong owner password, task resume | User-level install works without owner where safe; system change names owner requirement, can defer and resume; credential not shared with app/user history | Keyboard/Orca, separate owner assistive needs |
| J21 Offline day | “Work with your recent document and deal with anything that wants the internet later.” | No network from login, queued update, unavailable store/search | Local work remains available; no cascading alerts/spinners; tasks show offline state and retry/defer honestly | Keyboard/Orca, cognitive/interruption |
| J22 Undo and history | “Something changed yesterday and the printer stopped working. See what changed and undo only the relevant change.” | Multiple action records, expired/unsafe undo, time change | Finds human history, identifies provenance/scope, performs safe undo or receives honest alternative; audit reveals no unrelated private data | Keyboard/Orca, 200%, low literacy |

#### 25.1 Mandatory fault injection

For J08–J17 and J22, test at least these commit boundaries where applicable:

- before authorization; during decision; immediately after approval;
- before snapshot/undo creation; after snapshot but before change;
- during download; after download before verification; during apply;
- after partial mechanism work; while writing audit; before success response;
- app closes; session locks; user switches; broker/presenter/mechanism exits;
- disk becomes full; network disconnects; AC power is removed; and
- system restarts or loses power.

Each injected outcome must say whether state is unchanged, rolled back, safely
in progress, or partial. “Try again” is offered only if repetition is safe.

### 26. Analysis and decision process

Immediately after each session, observer and facilitator separately note the
top failures before discussion, then merge evidence. Within one working day the
team creates findings containing:

```text
Finding ID and title
Observed behavior (not interpretation)
Affected journey/build/context/access method
Participant count and contrary evidence
User consequence and security/accessibility invariant
Likely design cause (labelled hypothesis)
Severity
Recommended experiment/change and owner
Retest evidence required
```

Use affinity analysis across sessions, but preserve outliers with high impact.
Do not vote away a disabled participant's blocker because most participants did
not use that modality. Pair qualitative evidence with event/time data; neither
alone explains the cause.

At the end of a round:

1. fix critical/high issues or explicitly stop the affected release;
2. choose the few highest-risk assumptions for the next round;
3. update journey fixtures and expected invariants when the product contract
   legitimately changes;
4. record which finding each design change addresses; and
5. retest the actual end-to-end build, not only revised copy or mockups.

A finding closes only when new evidence shows the failure no longer occurs in
the affected contexts and no new safety/accessibility regression was introduced.

### 27. Summative reporting

The stable-release report must publish:

- build and supported-context definition;
- recruitment and exclusions;
- task wording and fixture states;
- outcome counts including TF and AB;
- per-journey safe-independent rate with confidence interval;
- assistance distribution, time, errors, terminal attempts, and ease;
- authorization comprehension and unsafe-decision results;
- results segmented by declared access modality, confidence, account role, and
  prior OS where sample permits without re-identification;
- all critical/high findings and disposition;
- known limitations and untested contexts; and
- changes made since the test and whether those changes were retested.

Do not claim “intuitive” from satisfaction alone, a five-person round with no
observed failure, or team members completing a checklist. The defensible claim
is always bounded: which people, goals, build, context, effectiveness,
efficiency, satisfaction, safety, and accessibility evidence.

---

## Part IV — Accessibility audit and conformance plan

### 28. Accessibility baseline

LNP targets independent completion rather than a paper-only conformance claim.
For native desktop UI, WCAG 2.2 AA is used as a testable minimum analogy—not a
claim that a web standard formally certifies the whole operating system—plus
KDE HIG requirements, AT-SPI platform semantics, and user research with
disabled people. Relevant WCAG 2.2 additions include unobscured focus, a
non-dragging alternative, minimum target size, consistent help, and accessible
authentication ([WCAG 2.2](https://www.w3.org/TR/WCAG22/)). KDE's HIG likewise
requires keyboard access and screen-reader semantics
([KDE accessibility and inclusiveness](https://develop.kde.org/hig/accessibility/)).

AT-SPI is the Linux desktop accessibility interface through which UI elements
expose role, name, state, relations, actions, text, focus, selection, and events.
Custom-rendered pixels do not become accessible because their labels are drawn
visibly. They need a synchronized semantic tree. `accesskit_unix`, for example,
adapts an AccessKit tree to AT-SPI
([AccessKit Unix](https://docs.rs/crate/accesskit_unix/latest)).

This baseline covers permanent, temporary, and situational impairment. A person
holding a child, using a laptop in sun, recovering with a broken mouse, or
reading a second language benefits from the same design even if they do not
identify as disabled.

### 29. Audit scope and evidence

The audit covers the complete supported journey, not only LNP-owned app
windows:

- boot, disk unlock, login, session start, lock, logout, and recovery;
- Plasma panel/menu/dock/window switching/notifications;
- Welcome/Computer Care and its authorization/progress/errors;
- Software, Settings, network, Bluetooth, display, device, and file chooser
  surfaces reached by LNP journeys;
- SELinux Security Alerts;
- guarded update and restart status;
- system/trusted authentication and future intent UI;
- Help, diagnostic export, and remote assistance; and
- fallback state when graphics, network, authentication, or assistive services
  fail.

Evidence for each component consists of:

1. a source/static review;
2. an AT-SPI tree and event inspection;
3. full keyboard and pointer/touch test;
4. visual adaptation checks;
5. at least one end-to-end automated accessibility smoke test where automation
   is technically possible;
6. manual test with the actual assistive technology; and
7. observed completion by representative users for critical journeys.

Passing only one layer is insufficient. An automation tool can activate a
nameless button by coordinates while a screen-reader user cannot identify it;
an expert can tab through controls that become unusable under real magnification.

### 30. Current repository audit (2026-08-08)

This is a static audit of the current source, not a runtime conformance claim.
Findings remain open until the stronger evidence above closes them.

| ID | Severity | Component and evidence | User impact | Required disposition |
| --- | --- | --- | --- | --- |
| A-001 | Critical | The Rust dock depends on SCTK, `tiny-skia`, and `fontdue` and contains no AccessKit/AT-SPI dependency or semantic-tree implementation (`dock/lnp-dock/Cargo.toml`, `src/main.rs`) | Screen readers and accessibility automation have no exposed model for launcher, apps, running state, menus, or tooltips | Do not stable-ship this implementation until it exposes a complete tested AT-SPI tree, or replace it with a native Plasma/Qt implementation |
| A-002 | Critical | The layer surface requests `KeyboardInteractivity::None`; keyboard handling for grabbed popups only implements Escape. App tiles, categories, dock items, and context actions have no keyboard navigation/activation (`dock/lnp-dock/src/main.rs`) | Keyboard, switch, speech, and screen-reader users cannot operate the central launcher/window path | Add a discoverable focus route, full roving/grid/menu navigation, activation/context actions, focus restoration, and visible focus; test through AT-SPI |
| A-003 | High | Seat code requests pointer and keyboard capability, not touch; primary interactions are pointer press/motion/release | Touch-only operation is absent and concurrent input is unverified | Implement touch semantics with cancel/slop/long-press rules or use toolkit controls; retain pointer and keyboard concurrently |
| A-004 | High | Pin reordering is drag-only and hover tooltips are the only persistent text labels for dock icons | People unable to drag lack an equivalent; hover is unavailable to keyboard/touch and increases recognition burden | Add Move left/right commands and keyboard menu; accessible names must never depend on tooltip; retain labels in launcher/search |
| A-005 | High | Dock visuals/text are custom raster output with hard-coded geometry and animation; no evidence they consume system font, high contrast, reduced-motion, color-scheme, or text-size preferences | Scaling, contrast, motion, localization expansion, and color adaptation may fail silently | Replace constants with platform settings, support reduced/no magnification, test 100–400%, theme/contrast/RTL, and use semantic state beyond color/dots |
| A-006 | High | The failed-login recovery path is a timed text-console menu with a 60-second automatic choice and a helper shell; no screen-reader, magnification, pointer, locale, or timeout extension is implemented (`recovery/lnp-recovery-prompt`) | A person who cannot read/use the console quickly may be unable to recover the computer at the moment accessibility is most needed | Build an accessible recovery surface with speech/large text/keyboard-layout support and pause; retain console only as a final fallback |
| A-007 | High | The proposed compositor consent document contains only visual HTML diagrams and general focus CSS; it does not design a protected AT semantic/invocation path, secret handling, slow input, or remote accessibility | Implementing it could create a security UI disabled people cannot use or an AT bypass malware can abuse | Supersede with Part II §16.10; security and accessibility reviewers must jointly approve a prototype before system installation |
| A-008 | Medium | Welcome uses standard Qt widgets, a positive foundation, but rich-text `QLabel` bodies carry list/structure, progress is textual stdout, and dynamic status/error announcements and explicit accessible names/relations are not tested (`welcome/lnp-welcome`) | Screen-reader reading order/status may be unclear; raw helper phrasing may overwhelm; buttons and page changes may not announce as intended | Inspect QAccessible/AT-SPI; replace structural rich text with semantic widgets where needed; announce status; set names/descriptions/buddies; automate and user-test |
| A-009 | Medium | Welcome has fixed 720×540 initial size and presents several optional maintenance pages in a linear wizard | Large text, magnification, small displays, fatigue, and cognitive load may cause clipping or unnecessary work | Make pages scroll/reflow, show only relevant setup, move optional work to persistent Computer Care, preserve progress and deferral |
| A-010 | Medium | The egui SELinux app transitively includes AccessKit (shown in `selinux/lnp-selinux/Cargo.lock`), but forces dark visuals and has no recorded AT-SPI/manual verification | A semantic bridge may exist, but theme/contrast and custom interaction could ignore user preferences; dependency presence is not conformance | Use system theme/contrast, inspect tree/actions/live events, test Orca and keyboard, add accessibility regression tests |
| A-011 | High | Security Alert notification text universally says “Nothing is broken and your files are safe” for each new alert | The message can contradict a blocked user task and makes a safety claim not established by the alert parser, undermining cognitive trust | Replace with outcome-specific structured copy; state only what is known and lead to one relevant action |
| A-012 | High | `lnp-desktop` does not require the separately packaged `lnp-selinux`, while `lnp-errord` no longer owns the SELinux experience | Default installs can omit the only intended graphical explanation/fix path, forcing technical diagnosis | Add the correct package dependency after removing/gating unsafe Tier 4 behavior and completing accessibility/security review |
| A-013 | Medium | The metapackage does not declare or verify the screen reader, speech, accessibility bus, on-screen keyboard, and recovery accessibility runtime needed by the promised experience | A clean supported install may lack the very tools required to enable accessibility independently | Define the supported accessibility stack, package it by default or on installation media, and verify from first boot without a terminal |
| A-014 | Medium | Notification/error handling is split across Python daemon stdout, Qt messages, systemd journal, console recovery, and egui notifications without a shared semantic/content schema | Severity, action, status announcement, and history can drift; assistive tech may receive duplicates or silence | Introduce structured event/error model and Computer Care history; map every event to visible and AT announcements consistently |

#### 30.1 Dock architecture decision

The current dock is a stable-release blocker because it is both the primary app
launcher/window return path and completely custom-rendered. Two implementation
spikes should be compared:

1. a Plasma/QML/Qt dock or plasmoid using native accessible controls and Plasma
   window models, extended only for the essential magnification/layout behavior;
2. the Rust layer-shell client with `accesskit_unix`, a full semantic/focus
   model, platform settings, touch, and automated AT-SPI tests.

Prefer the native implementation unless evidence shows it cannot meet a core
user need. Pixel-perfect magnification does not outweigh keyboard, screen-reader,
theme, localization, and integration correctness. If Rust remains, every visual
`Slot`, app tile, category, context item, badge, tooltip, and progress state must
have a stable accessible node/action/state and synchronized bounds. The tree
must update correctly when windows appear, reorder, change title, minimize, move
display, or disappear.

### 31. Component requirements

#### 31.1 Semantics and screen readers

- Every interactive and meaningful element exposes the correct role, concise
  accessible name, state/value, description only when needed, and available
  action through AT-SPI.
- The visible label is contained in the accessible name so speech users can say
  what they see. Icon-only controls have tested localized names.
- Groups, page titles, lists/grids, dialogs, alerts, progress, status, errors,
  and relationships are represented semantically rather than encoded only in
  drawing order or punctuation.
- Decorative icons/images are ignored. Informative images have equivalent text;
  complex state has a structured textual equivalent.
- Dynamic changes emit the correct AT-SPI events. Routine animation is not
  announced; completion, error, selected state, and newly required action are.
- Window/app names distinguish multiple documents without reading private
  content unnecessarily in public announcements.
- Orca speech and braille output is verified for each critical journey,
  including login, polkit, future trusted UI, notifications, and recovery.
- Secrets are never exposed to generic accessibility logs/history. A protected
  credential field speaks only the configured safe feedback.

#### 31.2 Keyboard, switch, and speech operation

- Every pointer/touch action has a keyboard route. Tab order follows visual and
  logical order; arrow keys operate menus, grids, radio groups, tabs, and
  spatial dock items according to platform convention.
- Focus is always visible at at least 3:1 contrast against adjacent colors,
  never wholly obscured, and remains meaningful after content updates.
- Opening a dialog/menu moves focus to a safe, relevant element; closing returns
  focus to its invoker. No keyboard trap exists except a true modal decision,
  where Cancel/Escape remains available.
- Escape cancels transient UI without discarding committed work. Enter/Space
  behavior matches the control role. Destructive actions are not defaulted to
  an accidental key repeat.
- Reordering and other drag actions have Move before/after/up/down commands.
- Global shortcuts are discoverable and remappable. Accessibility activation
  from boot/login does not require a terminal or prior sighted setup.
- Visible labels and accessible names match closely enough for speech commands.
  Duplicate labels include disambiguating context.
- Key repeat, sticky keys, slow keys, bounce keys, one-handed modifiers, and
  switch scanning do not cause duplicate authorization or irreversible work.

#### 31.3 Pointer, touch, and motor access

- Primary targets aim for at least 44×44 logical pixels; no target may be below
  the WCAG 2.2 AA 24×24 CSS-pixel analogy without sufficient spacing or an
  essential exception documented in the audit.
- Pointer activation occurs on release inside the target, allowing cancellation
  by moving away. Press is reserved for established menu behavior and cannot
  commit a dangerous operation.
- Touch has adequate slop, no hover dependency, predictable long-press behavior,
  and alternatives for multi-finger or path gestures.
- Concurrent keyboard, pointer, touch, and assistive input remain enabled; using
  one must not permanently hide the affordance for another.
- Controls do not require fine motor timing, precise drag, double click, or
  holding a button while moving unless a simpler equivalent is adjacent.
- Timeouts are absent unless essential. When essential, warn, allow extension or
  pause, and make expiry safe. Authorization expiry never auto-approves.

#### 31.4 Vision, scaling, color, and contrast

- Text and icons use platform fonts/metrics and remain readable at 200% text
  size without loss of content or function; critical/recovery UI is tested at
  400% magnification with reflow/panning that preserves focus.
- Windows fit the smallest supported display and account for on-screen keyboard,
  panels, magnifier regions, and long translations. Content scrolls; essential
  buttons are not clipped.
- Normal text contrast is at least 4.5:1, large text at least 3:1, and meaningful
  controls/graphics/focus boundaries at least 3:1. Disabled state remains
  distinguishable without becoming the only explanation.
- State, severity, running status, and selection never depend on color alone.
  Shape, text, position, icon, or announced state provides equivalence.
- System light/dark/high-contrast and color preferences are respected. Apps do
  not force a theme.
- Magnifier follows keyboard and AT focus, not only pointer. Popups and trusted
  prompts appear in the visible region without covering their focused control.

#### 31.5 Motion, flashing, and vestibular safety

- Honor the system reduced-motion setting. Dock magnification, panel movement,
  progress transitions, and workspace effects become instant or subtle fades.
- No content flashes more than three times per second or uses high-area
  red/contrast flashing.
- Motion never conveys the only state change. Animation can be stopped by
  leaving the trigger and cannot block interaction.
- Parallax, elastic overshoot, zoom, and auto-moving content are off in reduced
  motion and configurable where they are central to the dock aesthetic.

#### 31.6 Hearing and multimodal status

- Every sound cue has a visual and semantic equivalent; critical recovery and
  authorization never rely on sound.
- Captions/transcripts are available for any LNP tour/support media.
- Audio-device tests include a visible meter/state and do not play a startling
  sound without warning.
- Notifications respect system duration and are available in durable history
  after disappearance.

#### 31.7 Cognitive, language, and learning access

- One screen has one clear primary purpose. Optional technical detail is
  collapsed without hiding the primary action.
- Instructions use common words, short sentences, concrete objects, consistent
  order, and exact action labels. Icons supplement rather than replace words in
  consequential UI.
- The product never creates false urgency, shame, blame, or a security puzzle.
- A person can review a decision at their pace; nothing moves, resets, or times
  out merely because reading is slow.
- Progress survives closing, interruption, lock, and restart where the operation
  does. The next step is stated explicitly.
- Authentication permits password managers/paste where system policy allows
  and offers non-memory/cognitive-test alternatives, consistent with the intent
  of WCAG 2.2 accessible authentication.
- Help appears consistently and preserves the current task/context.

#### 31.8 Localization and internationalization

- All user-facing strings, accessible names, speech text, metadata, recovery
  copy, and trusted prompts are localizable. No sentence is assembled from
  fragments whose grammar assumes English.
- Layout supports right-to-left direction, locale formats, input methods, and
  keyboard layout selection before credential entry/recovery.
- UI allows at least 200% string expansion and tests representative long German,
  compact CJK, Arabic RTL, accented text, emoji, and mixed-direction paths.
- Technical identifiers remain copyable but never replace localized meaning.
  Screen readers receive correct language metadata for speech switching.

#### 31.9 Authorization and recovery

- The security level cannot be inferred from dimming/color alone; role,
  requester, action, scope, and button verbs are semantic and spoken.
- Trusted UI supports keyboard, screen reader, magnifier, alternative input,
  locale/layout, and slow interaction without delegating approval to an
  ordinary AT client.
- Password/fingerprint/security-key alternatives do not require solving a
  cognitive puzzle or transcribing an unpasteable secret.
- Cancel, denial, timeout, presenter crash, and AT failure change nothing.
- Recovery accessibility starts before the first recovery choice. Screen reader
  and magnification activation is discoverable on the surface and installation
  media.
- A helper path cannot be the only accessible path. Remote accessibility is an
  explicit capability/security design (§16.11), not screen-coordinate control
  smuggled around the trusted presenter.

### 32. Audit procedure

#### 32.1 Static and semantic inspection

For each window/state:

1. enumerate visual controls and meaningful content;
2. inspect the AT-SPI tree with Accerciser or equivalent;
3. compare role/name/state/actions/bounds/order with the visual model;
4. trigger dynamic updates and inspect focus/live events;
5. inspect source for coordinate-only controls, custom painting, hover/drag-only
   behavior, forced colors/fonts, timeouts, and unlocalized text; and
6. record screenshot, semantic-tree snapshot, build, scale/theme/locale, and
   finding IDs.

KDE recommends Accerciser and Orca alongside Appium accessibility tests
([KDE Appium testing](https://develop.kde.org/docs/apps/tests/appium/)). Automated
tests should select elements by accessible identity/action rather than screen
coordinates; doing so tests the interface assistive technology needs.

#### 32.2 Keyboard pass

With pointer/touch physically unavailable, start from boot/login and complete
every applicable core journey. Record every focus stop, missing action,
unexpected order, trap, focus loss, invisible/obscured focus, key conflict, and
operation triggered on press/repeat. Repeat with sticky/slow keys where relevant.

#### 32.3 Screen-reader pass

Use the supported Orca version with speech and, when available, braille. The
tester must be able to determine context, navigate efficiently, inspect state,
activate and cancel, hear dynamic outcomes, recover focus, and distinguish
multiple windows. Test from a clean account so success does not depend on
custom scripts or memorized coordinates.

#### 32.4 Visual adaptation pass

Run light, dark, high contrast, common color-vision simulations as a supplement
(never a substitute for users), 200% UI scale, 200% text, KWin magnifier up to
400%, small display, long localization, RTL, reduced motion, large pointer, and
on-screen keyboard. Capture clipping, overlap, off-screen dialogs, lost focus,
bitmap blur, and state distinctions.

#### 32.5 Input and timing pass

Test mouse, touchpad, touchscreen, keyboard, one alternative input used by a
participant, slow interaction, accidental movement, cancel-on-release, drag
alternatives, and device hotplug. Let notifications and recovery prompts sit
past nominal timeouts. Suspend/resume and lock/unlock during dynamic tasks.

#### 32.6 User pass

Run the journey cards in Part III with people who already use the relevant
assistive technology. Let them use personal strategies. The facilitator does
not force “correct” screen-reader commands. Record independence, fatigue,
workarounds, and whether the semantic structure matches the person's model.

### 33. Accessibility issue and gate model

Each finding records component/build, journey, impairment/access context,
reproduction from a clean account, expected semantic/visual behavior, actual
behavior, AT/version/input, severity, evidence, owner, fix, regression test, and
retest by an affected modality.

Severity uses Part III §24.3. Examples:

- missing dock accessibility tree, inaccessible trusted prompt, or inaccessible
  recovery is **Critical**;
- one critical journey lacking keyboard operation is **Critical/High** depending
  on whether a platform-equivalent route exists;
- clipped but scroll-reachable secondary details at 200% may be **Medium**;
- a slightly inefficient but correctly spoken grouping may be **Low**.

Stable release requires:

- no open Critical or High findings in LNP or the supported upstream path;
- 100% critical-journey completion for each applicable supported modality in
  the release audit fixtures;
- semantic-tree regression coverage for every LNP-owned interactive surface;
- manual Orca and keyboard evidence for each release candidate;
- trusted and recovery UI reviewed jointly by security and accessibility
  specialists;
- representative disabled-user research in the current major interaction
  design, not only an older prototype; and
- documented upstream exceptions with mitigation, affected journey, public
  limitation, owner, and removal target. A terminal workaround is never an
  acceptable mitigation for a critical normal journey.

---

## Part V — Delivery plan and traceability

### 34. Required work sequence

Security, accessibility, and usability work cannot be serialized as “build,
then audit.” The safe sequence is:

#### Phase 0 — Adopt the product contract

- Make this specification the design/release authority and mark the legacy
  consent draft as historical.
- Rename claims such as “updates that cannot hurt you” to “guarded updates” and
  audit every absolute safety claim.
- Define supported Fedora/KDE/hardware/accessibility context and name release
  owners.
- Create evidence templates for feature gates, research findings, security
  review, and accessibility findings.

**Exit:** maintainers agree on north star, non-goals, risk ladder, critical
journeys, and stable-release gates.

#### Phase 1 — Remove current high-risk gaps

- Replace `org.lnp.setup` and generic `lnp-setup` with narrow typed operations
  and action-specific policy; remove `auth_admin_keep`.
- Remove the normal `allow-module` action and ensure one selected SELinux
  incident cannot include unrelated denials.
- Add `lnp-selinux` to the metapackage only after its safe remaining operations
  and accessible UI pass their gates.
- Decide and implement the dock accessibility architecture; add full keyboard,
  touch, semantic, theme/scale, and reduced-motion support.
- Replace universal reassurance and raw stdout presentation with structured
  result/error data.
- Define/package first-boot accessibility and begin accessible recovery design.

**Exit:** no Critical known blocker in the normal shell; no broad retained root
authority; Tier 4 SELinux generation is absent from ordinary UI.

#### Phase 2 — Build the durable experience

- Evolve Welcome into relevant onboarding plus persistent Computer Care.
- Implement structured task/progress/history/undo state and contextual Help.
- Split cleanup, security, media, driver, update, and recovery jobs according to
  the risk table.
- Harden package-source metadata, file-descriptor path handling, transactions,
  audit, undo, and fault behavior.
- Build the accessible recovery surface and diagnostic/support bundle.

**Exit:** J01–J12 and J15–J22 run end-to-end on supported fixtures with no
terminal and pass the accessibility component gates.

#### Phase 3 — Iterate with people

- Run discovery/context work before finalizing Computer Care information
  architecture.
- Test 4–8 people each round, including access needs, at least every two weeks
  while critical flows change.
- Fix and retest critical/high findings; baseline against current Fedora KDE and
  previous LNP build.
- Stabilize content vocabulary and supported hardware claims from evidence.

**Exit:** three consecutive rounds have no repeated Critical/High normal-journey
failure, and every critical assumption has direct evidence or a documented
release limitation.

#### Phase 4 — Upstream intent authorization

- Write a standalone upstream protocol/threat-model proposal from Part II with
  KDE, GNOME, polkit, portal, PAM, accessibility, and distribution stakeholders.
- Prototype the authority, typed request lifecycle, root-owned manifest, and
  compositor presentation without system installation.
- Commission security review of identity, same-UID/compositor boundary,
  replay/race, spoofing, capture/input, and mechanism isolation.
- Co-design and test protected AT semantics/input and remote-assistance policy.
- Only after review, pilot one genuinely Tier 2, bounded, reversible operation;
  do not begin with repository, driver, or policy installation.

**Exit:** upstream-accepted or independently maintainable architecture with a
published security boundary, accessible reference implementation, adversarial
tests, and no semantic misuse of authentication APIs.

#### Phase 5 — Stable qualification

- Run the complete security verification plan, accessibility audit, hardware
  matrix, and fault-injected core journeys on clean release images.
- Run a pre-registered summative benchmark and publish bounded results.
- Resolve every gate or document an honest supported-context limitation; remove
  unsupported code/claims from the default install.
- Rehearse updates, rollback, support, and recovery from the shipped image.

**Exit:** all §9 product gates, §17 security evidence, §27 research reporting,
and §33 accessibility gates are satisfied.

### 35. Current repository traceability

| Requirement | Current evidence | Status |
| --- | --- | --- |
| Preserve user choices and layout undo | `apply/lnp-apply-layout` checks unset touchpad state, backs up Plasma config, and supports revert | Partial: strong direction; runtime journey/accessibility testing missing |
| Narrow per-operation authorization | `welcome/org.lnp.setup.policy` has one action and `auth_admin_keep`; `lnp-setup` dispatches four broad subcommands | Contradicted; Phase 1 blocker |
| Selected SELinux scope | `selinux/lnp-selinux-fix allow-module` reads all recent AVC/USER_AVC records | Contradicted; Tier 4 action must leave normal UI |
| No retained authorization for SELinux | `selinux/org.lnp.selinux.policy` uses `auth_admin` rather than `_keep` | Meets narrow retention requirement, but one action still spans different risk tiers |
| Guarded update and rollback | `guard/lnp-guard` snapshots before `dnf upgrade`; `recovery/lnp-restore` preserves replaced root | Partial: important mechanism exists; broad claims, topology assumptions, transaction/fault tests, UI/accessibility incomplete |
| Recovery without terminal | `recovery/lnp-recovery-prompt` recommends restore and relegates shell to helper path | Partial: good information hierarchy; inaccessible timed console and unauthenticated policy unresolved |
| Actionable notification restraint | `errord/lnp-errord` rate-limits and filters routine unit churn | Partial: good principle; shared structured event model and end-to-end evidence missing |
| Accessible primary shell | Custom Rust dock has pointer-rendered controls and no semantic accessibility dependency | Contradicted; Critical blocker |
| Accessible LNP apps | Welcome uses Qt; SELinux's eframe dependency graph includes AccessKit | Unproven; must inspect runtime trees and complete manual/user tests |
| Default graphical SELinux path | Separate `lnp-selinux` package exists; `lnp-desktop` does not require it | Contradicted after errord ownership change; add only after safety remediation |
| Honest consent architecture | Legacy draft proposes consent through authentication response and a KWin trust claim without full AT/lifecycle design | Superseded by Part II; do not implement as written |
| Product claims match current state | README status says welcome/error features are “still to come” although implementations exist; spec says “updates that cannot hurt you”; URLs use `example.invalid` | Contradicted/stale; documentation and release metadata need reconciliation |

### 36. Decision records required before implementation

The following decisions cannot be silently buried in code:

1. supported hardware/filesystem/session and accessibility stack;
2. native Plasma/Qt versus custom Rust dock after the accessibility spike;
3. owner/standard-account setup and recovery authorization policy;
4. package/source trust policy for media and proprietary drivers;
5. automatic versus user-triggered update policy and snapshot limitations;
6. structured action broker/mechanism boundary and per-action manifest format;
7. audit retention, redaction, diagnostic bundle, and optional research data;
8. remote assistance capability model;
9. upstream ownership and security-boundary definition for trusted intent; and
10. any release-gate exception, with affected users and removal plan.

Each decision record contains context, user need, options, evidence, security
and accessibility effects, choice, rejected alternatives, test plan, owner, and
review date.

### 37. Definition of complete for this design

This design package is complete when it supplies, in one authority:

- an enforceable product doctrine and north-star measure;
- target people, jobs, boundaries, interaction/content/recovery rules, and
  release gates;
- a threat model with assets, actors, trust boundaries, invariants, risk tiers,
  current-operation mapping, present-safe path, future architecture, rejected
  designs, and security tests;
- a repeatable research plan with recruitment, ethics, fixtures, protocol,
  scoring, 22 core journeys, fault injection, analysis, and reporting;
- an accessibility baseline, current-source audit, modality requirements,
  audit procedure, issue model, and stable gate; and
- a phased implementation plan traceable to current repository evidence.

Implementation conformance is a separate claim. The document deliberately says
“proposed” until the current blockers in §30 and §35 are resolved and the
specified evidence exists.

### 38. Research and platform references

#### Product and platform guidance

- [ISO 9241-11:2018 — Usability: Definitions and concepts](https://www.iso.org/standard/63500.html)
- [NIST Human-Centered Cybersecurity](https://csrc.nist.gov/Projects/human-centered-cybersecurity/about)
- [KDE Human Interface Guidelines](https://develop.kde.org/hig/)
- [KDE — Powerful when needed](https://develop.kde.org/hig/powerful_when_needed/)
- [KDE — Accessibility and inclusiveness](https://develop.kde.org/hig/accessibility/)
- [Apple Human Interface Guidelines — Alerts](https://developer.apple.com/design/human-interface-guidelines/alerts)
- [Apple Human Interface Guidelines — Privacy](https://developer.apple.com/design/human-interface-guidelines/privacy/)
- [Apple Human Interface Guidelines — Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility)
- [Microsoft — Windows writing style](https://learn.microsoft.com/en-us/windows/apps/design/style/writing-style)
- [Microsoft — Dialog controls](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/dialogs)
- [W3C Web Content Accessibility Guidelines 2.2](https://www.w3.org/TR/WCAG22/)
- [Nielsen Norman Group — Ten Usability Heuristics](https://media.nngroup.com/media/articles/attachments/Heuristic_Summary_compressed.pdf)

#### Authorization and platform security

- [Microsoft — User Account Control: how it works](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/how-it-works)
- [Microsoft — Administrator Protection](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/administrator-protection/)
- [Apple Authorization Services](https://developer.apple.com/documentation/security/authorization-services)
- [Apple Platform Security — Controlling app access to files](https://support.apple.com/guide/security/controlling-app-access-to-files-secddd1d86a6/web)
- [polkit reference manual](https://polkit.pages.freedesktop.org/polkit/polkit.8.html)
- [XDG Desktop Portal documentation](https://flatpak.github.io/xdg-desktop-portal/docs/)
- [XDG Desktop Portal request model](https://flatpak.github.io/xdg-desktop-portal/docs/requests.html)
- [Wayland architecture](https://wayland.freedesktop.org/docs/html/ch03.html)

#### Human factors and usable security

- [Felt et al. — Android Permissions: User Attention, Comprehension, and Behavior](https://research.google/pubs/android-permissions-user-attention-comprehension-and-behavior/)
- [Bravo-Lillo et al. — Your Attention Please: Designing Security-Decision UIs](https://www.microsoft.com/en-us/research/publication/your-attention-please-designing-security-decision-uis-to-make-genuine-risks-harder-to-ignore/)
- [Vance et al. — The Fog of Warnings](https://www.usenix.org/conference/soups2019/presentation/vance)
- [Sunshine et al. — Crying Wolf: An Empirical Study of SSL Warning Effectiveness](https://www.cs.cmu.edu/~halmuhim/crying_wolf_ssl_browsers_usenix_2009.pdf)
- [Cranor — A Framework for Reasoning About the Human in the Loop](https://www.usenix.org/conference/upsec-08/framework-reasoning-about-human-loop)
- [Adams and Sasse — Users Are Not the Enemy](https://discovery.ucl.ac.uk/id/eprint/20247/)
- [Motiee et al. — Do Windows Users Follow the Principle of Least Privilege?](https://cups.cs.cmu.edu/soups/2010/proceedings/a1_motiee.pdf)

#### Research practice

- [GOV.UK — User research for government services](https://www.gov.uk/service-manual/user-research/how-user-research-improves-service-design)
- [GOV.UK — Plan user research for your service](https://www.gov.uk/service-manual/user-research/plan-user-research-for-your-service)
- [GOV.UK — Plan a round of user research](https://www.gov.uk/service-manual/user-research/plan-round-of-user-research)
- [GOV.UK — Find user research participants](https://www.gov.uk/service-manual/user-research/find-user-research-participants)
- [GOV.UK — Making your service accessible](https://www.gov.uk/service-manual/helping-people-to-use-your-service/making-your-service-accessible-an-introduction)
