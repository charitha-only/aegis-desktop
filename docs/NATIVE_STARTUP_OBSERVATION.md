# Windows native startup observation — 2026-11-05

Source: b0d9e79d9e543d1c4df19f436ec1c46c213658db. This is an unresolved intermittent startup observation, not a confirmed SDK regression or a fixed native bug.

## Observed failure

[Initial Windows native job](https://github.com/charitha-only/aegis-desktop/actions/runs/41742778773/job/126288558441) passed its 106-test native suite and explicit clipboard fixture, then failed native-smoke.js. The primary document produced no logged title event before the bounded deadline. The runtime reported CHILDREN_DROPPED=0 and 'Native fixture document did not load before timeout'. The launcher failed rather than accepting readiness. Linux/macOS native jobs passed; all three portable/template/SDK jobs and Windows installer checks passed.

## Controlled retry

Only the failed Windows job was rerun, without source edits or deadline changes. [Retry job](https://github.com/charitha-only/aegis-desktop/actions/runs/41742778773/job/127285244594) passed the 106-test native suite, explicit clipboard fixture and complete actual-window capacity/recovery/live-child teardown fixture. The GUI step completed in about 14 seconds. Workflow attempt 2 is green. A source comparison against the earlier passing ef6d1a6 contains no changes to src-core, native-smoke.js or aegis.config.json. SDK changes therefore did not modify the native adapter/fixture source. This comparison does not prove identical binaries, toolchains, runner load or WebView versions.

## Interpretation and remaining work

The retry did not reproduce the timeout. The available logs do not identify the cause; runner/WebView initialization delay, event-loop startup timing and fixture loading remain hypotheses. No timeout was raised, no assertion was weakened and no speculative native patch was applied. One retry passing is not a reliability guarantee or proof the first failure was harmless.

If it recurs, collect narrow timestamps around native environment creation, primary object creation and page-load/title transitions on disposable runners, then repeat/minimize the same public GUI fixture. Retain both failing and passing runs. Do not use personal-device GUI actions or label the issue fixed solely because a retry passes. Physical acceptance and startup race/fault coverage remain open release gates.


## Observation hardening follow-up — pending hosted verification

The native smoke now accepts only --repeat=1 or --repeat=3 after its hosted-runner guard. Windows CI requires three sequential fresh-fixture runs in the same job; macOS/Linux retain one. Each iteration owns a fresh temporary project and WebView data directory, requires the same readiness/one-child-release assertions, and completes cleanup before the next. Any failure aborts the job; later success cannot replace a failed iteration. These are fresh fixtures on one runner, not independent physical machines or a statistical reliability estimate.

Windows/macOS smoke mode now emits monotonic elapsed milliseconds at host entry, event-loop/WebContext creation, resume, primary native-window creation, WebView construction, primary visibility, primary page-load start/finish, readiness and deadline. Stages contain no URL, nonce or application payload. This retained CI observability distinguishes native initialization delays from later document-loading stalls if the issue recurs. It is gated by the existing hosted-runner smoke checks and does not emit telemetry in normal apps. The 45-second runtime and 60-second launcher deadlines are unchanged.

Local native cargo check and JavaScript syntax checks passed. No local GUI was run. Hosted results for the new repeat/timing source remain pending. This instrumentation is not a root-cause fix for the earlier timeout; its cause remains unresolved.
