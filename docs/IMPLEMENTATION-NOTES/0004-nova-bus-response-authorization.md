# 0004: Response-spoofing vulnerability found and fixed

Date: 2026-07-18
Status: Resolved — code fixed, regression test added, this note is the permanent record

## Architecture

[RFC-0002](../rfcs/RFC-0002-nova-bus.md) §Security Considerations states the broker's
ACL check is "the single enforcement point for the entire permission model" but, as
originally written, only describes authorization for the *initiating* side of a message
(`Call`, `Publish`, `Subscribe`, `RegisterHandler`) — it never enumerated a threat model
for the *responding* side of a `Call`/`Response` pair.

## Reality

The original `Command::Response` handler in
`services/nova-bus-broker/src/broker.rs` routed any `Response` bearing a known
`request_id` back to that request's original caller — without checking that the
connection sending the `Response` was the same connection the broker had actually
forwarded the corresponding `Call` to. Any connected client that observed or guessed a
`request_id` (broker-assigned, sequential — trivially guessable, see
[0005](0005-nova-bus-measured-performance.md)'s throughput numbers for how many are
issued per second under load) could inject a forged `Response` and have it delivered to
the real caller as if it came from the legitimate handler.

## Reason

Found by direct inspection while writing the `Command::Response` match arm during Phase
2 implementation — the `connection_id` field on `Command::Response` was constructed
(from the dispatching connection) but never read, which `cargo build` flagged as dead
code. Investigating *why* it was unused surfaced that nothing was using it to check
authorization.

## Decision

1. Added `handler_connection: u64` to `PendingCall`, recorded when the broker forwards
   a `Call` to its registered handler.
2. `Command::Response` now checks `pending.handler_connection == connection_id` before
   honoring the response; a mismatch is dropped with a `warn!`-level log (not silently
   ignored — a spoofing attempt is a security-relevant event) and the genuinely pending
   call is left untouched (peek-then-remove, not remove-then-maybe-restore, so a forged
   attempt can never invalidate the real handler's still-pending response).
3. Added `a_response_from_the_wrong_connection_is_dropped_not_routed` — a test that
   forges a `Response` from an unrelated third connection, asserts the caller receives
   nothing, then asserts the *genuine* handler's later response still succeeds normally
   (proving the forged attempt didn't corrupt the pending-call state).

## Future Direction

None required — fully closed, including amending
[RFC-0002](../rfcs/RFC-0002-nova-bus.md) §Security Considerations to state the Response
authorization requirement explicitly (done in the same change as this note, see that
RFC's Changelog) rather than leaving it implicit in the code alone. Recorded here as a
concrete example, for Phase 2.5 System Validation's "is Nova Bus actually the right
abstraction" question
([12-ROADMAP-AND-MILESTONES.md](../12-ROADMAP-AND-MILESTONES.md) §4a), of the review
process already working during implementation, not only during the dedicated review
phase.
