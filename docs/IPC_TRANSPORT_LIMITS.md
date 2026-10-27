# Local HTTP transport limits

The IPC listener remains loopback-only. Host/origin checks, bearer validation,
JSON/content-length restrictions and request rate limiting remain in effect.
They do not protect a synchronous handler that is waiting for missing body bytes.

## Confirmed issue and remediation

Before v0.25.14, sending a valid POST header with `Content-Length: 65536` and only
one body byte blocked the single IPC handler. A separate handshake timed out after
eight seconds. This is a local availability issue; no authentication or vault
decryption bypass was demonstrated. The regression uses its own ephemeral core
process, port and temporary application-data directories.

The patched transport now enforces:

- An absolute five-second TCP read budget from accept, covering headers, small
  pre-buffered bodies, handler body reads and unread-body draining. Sending more
  bytes does not renew the budget. Socket clones preserve the same deadline.
- A separate absolute five-second response-write budget from the first write.
  Command computation before that write does not consume the response budget.
- One request per connection with `Connection: close`. Clients must reconnect;
  keep-alive reuse and pipelining are deliberately unavailable in this preview.
- At most 32 admitted TCP connections per listener. Shared permits remain held
  by socket clones and queued requests until their last owner is dropped. Excess
  connections are closed before allocating a connection worker.
- HTTP lines at most 8 KiB, at most 100 header fields and at most 64 KiB of header
  fields including line delimiters. Oversized input is rejected without waiting
  for the whole request.
- An 8 KiB scratch buffer for draining rejected bodies. The upstream destructor
  allocated a buffer based on the advertised remaining Content-Length, even when
  Aegis had already rejected that length. This source-confirmed allocation hazard
  was fixed without deliberately exhausting the user's PC. The fixed fixture
  advertises 4 GiB minus one byte and checks subsequent server availability.

## Dependency maintenance

`src-core/vendor/tiny_http` contains the tiny_http 0.12.0 source and original
MIT/Apache licenses. `AEGIS_PATCH.md` records the upstream archive SHA-256 and
modified files. Generated projects include this directory and its licenses.
Registry advisory scans do not review local modifications; review the patch and
rebase it explicitly during dependency upgrades.

## Verification and limits

After building the optimized core, run:

```text
node aegis-cli/bin/ipc-deadline-smoke.js
node aegis-cli/bin/portable-smoke.js
node --test aegis-cli/test/create.test.js
```

The transport fixture covers a trickling incomplete body, incomplete headers,
small buffered bodies, oversized rejected bodies, oversized header input,
connection admission and recovery after releasing slots. Portable CI runs it
on Windows, Linux and macOS. It does not lock or suspend the user's computer.

These bounds do not cancel a long-running command, undo committed side effects,
or guarantee availability against sustained same-machine flooding. A valid
synchronous command can still occupy the handler. OS scheduling, maximum live
connection buffers and complete process-tree RAM/CPU need physical low-resource
measurement. TLS and Unix-domain sockets are not Aegis IPC transports and are
outside this patch's contract. This is not independent audit certification.
