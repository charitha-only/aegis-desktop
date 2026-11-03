# Managed-window acceptance — capacity follow-up

Validated native source: ef6d1a6c45c2a145e1624376e0eef7ac99eda268. Date recorded: 2026-11-04 (Asia/Colombo).

The [native workflow](https://github.com/charitha-only/aegis-desktop/actions/runs/41735933346) passed on hosted Windows MSVC, macOS and Ubuntu. Ubuntu uses a disposable virtual display. Each PASS below refers to the actual guarded native-smoke.js fixture, not a mocked native provider.

| Observable condition | Windows | macOS | Ubuntu |
| --- | --- | --- | --- |
| Create primary and complete its scripted document checks | PASS | PASS | PASS |
| Create two children, list their distinct IDs, close and reopen | PASS | PASS | PASS |
| Reject another authority and an unknown close ID | PASS | PASS | PASS |
| Fill eight child slots and reject the ninth | PASS | PASS | PASS |
| Preserve all eight IDs after rejecting overflow | PASS | PASS | PASS |
| Free a slot and reuse capacity with a fresh ID | PASS | PASS | PASS |
| Keep one child registered before scripted host exit | PASS | PASS | PASS |
| Exit successfully and release exactly one retained child registry entry | PASS | PASS | PASS |

The last assertion requires both fixture readiness and the runtime's smoke-only CHILDREN_DROPPED=1 marker. It observes the common native-host cleanup path and owned-object release. The test triggers scripted event-loop exit, not a physical click on the primary close button. Child creation confirms native object allocation; child document readiness and rendering are not asserted. No evidence here establishes termination of every browser subprocess.

Not covered: user-operated close buttons, close/create races, in-flight shutdown cancellation, modal parenting, focus/minimize events, renderer failure, keyboard/screen-reader behavior, constrained-device budgets, physical displays, abrupt OS/process/power loss or storage durability. Windows GNU native execution remains unvalidated. These gaps remain release gates; passing the fixture does not certify sensitive applications.

The v0.29.18 tag remains pinned to its earlier source 3a61048. These expanded assertions apply to the later follow-up source ef6d1a6; the tag was not moved.


The same source also passed [three-OS portable/template/package verification](https://github.com/charitha-only/aegis-desktop/actions/runs/41735933330) and [Windows installer/upgrade/backup-preservation/real-lock verification](https://github.com/charitha-only/aegis-desktop/actions/runs/41735933316). The verification ledger records test counts, intentional skips and the remaining release gates.
