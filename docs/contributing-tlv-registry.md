# Contributing to the TLV Registry

## Overview

TLV Type IDs are **append-only forever**. Once allocated, never reused or reordered (Constitution IV — "Old URLs decode forever").

## TLV Type Ranges

The canonical source-of-truth lives in [`packages/codec/REGISTRY.md`](../packages/codec/REGISTRY.md). Summary:

| Range | Purpose | Status |
|-------|---------|--------|
| 1–13 | v1 core fields | LOCKED (Constitution IV) |
| 14 | ITEMS | LOCKED |
| 15–99 | VoidPay canonical core | v2 extensions (mandatory=even, optional=odd) |
| 100–199 | Agent-economy extensions | parentHash, budgetCap, delegationScope, split |
| 200–999 | Reserved canonical | future on-chain anchors, lifecycle, privacy |
| 1000–9999 | Vendor namespace | PR-merged FCFS |
| 10000+ | Experimental / reclaimable | 12-month inactivity policy |

## BOLT12 odd/even rule

- **Even** TLV types are mandatory — unknown even type → decode error
- **Odd** TLV types are optional — unknown odd type → ignore and pass through

This enables forward compatibility: future codecs add odd TLV types that older decoders skip cleanly.

## How to Allocate

1. **Pick a range** matching your use case (see table above)
2. **Open a PR** titled `[TLV] allocate <range>:<type-id> for <feature>`
3. **PR body MUST include**:
   - Motivation (why this allocation, who uses it)
   - Encoding spec (byte-level layout: type → length → value)
   - Backward-compatibility statement (does it break v1 decoders?)
   - Test vector (encoded hex + decoded JSON example)
4. **Vendor namespace allocations** MUST use `vendor.<orgname>.<feature>` sub-key convention to prevent collisions
5. **Editor reviews PR and merges** when well-formed

## Vendor Squatting Policy

Per spec §4.4: **12-month inactivity reclaim**. Unused vendor allocations may be reclaimed by editors after 12 months of no on-chain or on-protocol activity, with 30-day PR notice on the registry.

## Editor Role

Per EIP-1 verbatim: **administrative only**. Editors merge well-formed PRs that respect ranges and naming. They do NOT pass judgment on feature value.

Magicians-style governance forum is deferred until ≥3 external implementations exist (premature governance theatre at our scale).

## Phase 1 Editor

- @ignromanov

## Future (Phase 3+)

When the codec ecosystem matures, editor responsibilities migrate to a `@void-layer/maintainers` team.

## References

- Spec 056 §4.4 (TLV Registry — BOLT-Style Federated)
- [`packages/codec/REGISTRY.md`](../packages/codec/REGISTRY.md) — canonical type-ID source-of-truth
