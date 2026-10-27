# Aegis local transport patch

Base: crates.io tiny_http 0.12.0 (MIT OR Apache-2.0). Original archive SHA-256: `389915df6413a2e74fb181895f933386023c71110878cd0825588928e64cdc82`.

Changes are restricted to `src/connection.rs`, `src/util/equal_reader.rs`, `src/client.rs`, `src/response.rs` and `src/lib.rs`:
- Bound each HTTP request/header line to 8 KiB and the header section to 100 fields / 64 KiB. Oversized input is rejected before reading the remainder.
- Admit at most 32 live TCP connections per server before creating connection work; shared permits follow socket clones and queued requests until their last owner is dropped. Overload closes the newly accepted socket.
- Serve one request per connection and advertise `Connection: close`, avoiding pipeline queue growth and stale keep-alive reuse.
- TCP reads have a shared, absolute five-second budget from accept, including header/body/drain reads. Socket clones preserve the deadline. Clients reconnect for each request.
- Response writes share a separate five-second absolute budget starting with the first write. Slow readers cannot renew it. Long command computation does not consume the first response-write budget.
- Rejected-body draining uses at most an 8 KiB scratch buffer, irrespective of advertised Content-Length.

This is a maintained local fork: registry advisory scanning does not audit these modifications. Preserve upstream licenses, review the patch and rebase it when updating upstream. Unix-domain sockets and TLS are not used by Aegis and are outside this patch's contract. Command computation deadlines and physical resource measurements remain separate work; these deadlines do not establish resistance to sustained local flooding.
