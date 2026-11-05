# Windows native startup observation — 2026-11-05

Source: b0d9e79d9e543d1c4df19f436ec1c46c213658db. This is an unresolved intermittent startup observation, not a confirmed SDK regression or a fixed native bug.

## Observed failure

[Initial Windows native job](https://github.com/charitha-only/aegis-desktop/actions/runs/41742778773/job/126288558441) passed its 106-test native suite and explicit clipboard fixture, then failed native-smoke.js. The primary document produced no logged title event before the bounded deadline. The runtime reported CHILDREN_DROPPED=0 and 'Native fixture document did not load before timeout'. The launcher failed rather than accepting readiness. Linux/macOS native jobs passed; all three portable/template/SDK jobs and Windows installer checks passed.

## Controlled retry

Only the failed Windows job was rerun, without source edits or deadline changes. [Retry job](https://github.com/charitha-only/aegis-desktop/actions/runs/41742778773/job/127285244594) passed the 106-test native suite, explicit clipboard fixture and complete actual-window capacity/recovery/live-child teardown fixture. The GUI step completed in about 14 seconds. Workflow attempt 2 is green. A source comparison against the earlier passing ef6d1a6 contains no changes to src-core, native-smoke.js or aegis.config.json. SDK changes therefore did not modify the native adapter/fixture source. This comparison does not prove identical binaries, toolchains, runner load or WebView versions.

## Interpretation and remaining work

The retry did not reproduce the timeout. The available logs do not identify the cause; runner/WebView initialization delay, event-loop startup timing and fixture loading remain hypotheses. No timeout was raised, no assertion was weakened and no speculative native patch was applied. One retry passing is not a reliability guarantee or proof the first failure was harmless.

If it recurs, collect narrow timestamps around native environment creation, primary object creation and page-load/title transitions on disposable runners, then repeat/minimize the same public GUI fixture. Retain both failing and passing runs. Do not use personal-device GUI actions or label the issue fixed solely because a retry passes. Physical acceptance and startup race/fault coverage remain open release gates.


## Observation hardening follow-up — verified 2026-11-05

The native smoke now accepts only --repeat=1 or --repeat=3 after its hosted-runner guard. Windows CI requires three sequential fresh-fixture runs in the same job; macOS/Linux retain one. Each iteration owns a fresh temporary project and WebView data directory, requires the same readiness/one-child-release assertions, and completes cleanup before the next. Any failure aborts the job; later success cannot replace a failed iteration. These are fresh fixtures on one runner, not independent physical machines or a statistical reliability estimate.

Windows/macOS smoke mode now emits monotonic elapsed milliseconds at host entry, event-loop/WebContext creation, resume, primary native-window creation, WebView construction, primary visibility, primary page-load start/finish, readiness and deadline. Stages contain no URL, nonce or application payload. This retained CI observability distinguishes native initialization delays from later document-loading stalls if the issue recurs. It is gated by the existing hosted-runner smoke checks and does not emit telemetry in normal apps. The 45-second runtime and 60-second launcher deadlines are unchanged.

Local native cargo check and JavaScript syntax checks passed. No local GUI was run. Exact source 829411d passed native workflow 42040446975 (Windows three runs, macOS/Linux one each), portable/template workflow 42040446956 and Windows installer/upgrade/data-preservation/real-lock workflow 42040446989, all on attempt 1. This instrumentation is not a root-cause fix for the earlier timeout; its cause remains unresolved.


## First timed native result — source 829411d

[Native workflow 42040446975](https://github.com/charitha-only/aegis-desktop/actions/runs/42040446975) passed on Windows/macOS/Linux. Windows required all three runs in one job, with no failed-iteration retry. macOS/Linux required one each. The earlier timeout did not recur in this small sample; its cause is still unresolved.

Windows stage observations (milliseconds):

| Iteration | WebView build begins after host entry | WebView created after host entry | Build interval | Primary load finished after host entry | Entire fixture ready after host entry |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | 13 | 8148 | 8135 | 8417 | 12638 |
| 2 | 11 | 340 | 329 | 435 | 4630 |
| 3 | 12 | 333 | 321 | 855 | 5053 |

Each iteration had a fresh project/WebView data directory on the same hosted machine. Shared runtime/OS cache state was not controlled. The first iteration's longer WebView construction is an observation, not proof of the earlier timeout's cause. Do not label these rows cold-boot benchmarks or independent hardware trials. Host entry excludes earlier core/process initialization; load-finished is not first paint, and fixture-ready includes the child-capacity/teardown checks. No whole-app responsiveness, low-resource-device or memory budget is established here.


The primary_visible marker denotes that the native show API returned, not physical visibility or first-paint confirmation. This diagnostic source does not raise deadlines, retry failed iterations automatically or establish a root-cause fix for the original timeout.
