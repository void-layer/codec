# TLV Registry — @void-layer/codec

> Canonical source-of-truth for TLV type range allocations.
> Governance model: BOLT-style federated (GitHub PR-driven, FCFS for vendor namespace).
> Per spec 056 §4.4.

## TLV Type Ranges — Invoice Message Kind (0x01)

```
1–13       v1 core fields              [LOCKED — Constitution IV]
14         ITEMS                       [LOCKED]
15–99      VoidPay canonical core      [v2 core extensions; mandatory=even, optional=odd]
100–199    Agent-economy extensions    [parentHash, budgetCap, delegationScope, split, ...]
200–999    Reserved canonical          [future on-chain anchors, lifecycle, privacy disclosure]
1000–9999  Vendor namespace            [vendor.<orgname>.* — PR-merged FCFS]
10000+     Experimental / reclaimable  [12-month inactivity → reclaim policy]
```

## Vendor Namespace Governance

- Vendor entries follow the naming convention: `vendor.<orgname>.<feature>`
- Allocation is first-come-first-served via GitHub PR
- Editor role is administrative only (per EIP-1 verbatim — editors don't pass judgment)
- Magicians-style forum deferred until ≥3 external implementations exist

## Vendor Squatting Reclaim Policy

Any vendor namespace entry (type range 1000–9999) that has had **no activity** (no merged PR, no published release referencing the type, no tracked usage) for **12 consecutive months** is eligible for reclamation. Reclaim process:

1. Maintainer opens a GitHub issue tagging the original allocatee
2. 30-day comment period
3. If no response: PR removes the vendor entry; type ID becomes available for re-allocation
4. Original allocatee may re-request the same ID within 90 days of reclaim if they demonstrate active use

## Per-record Allocation

Each spec's PR proposes specific Type IDs in the appropriate range:
- Spec 057 (lateFee) PR → proposes Type in 100–199 range
- Spec 060 (agent) PR → proposes parentHash, budgetCap etc. in 100–199
- Spec 064 (contract) PR → proposes contractAddress, proofIndex in 200–999

## Allocated Entries

_No entries yet. Phase 1 scaffolding._
