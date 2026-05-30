# Frozen Oracle Provenance Record

> **Immutable audit record.** Do not edit without re-running the provenance proof
> and updating this file + the `content_hash` in `v4-codec.json`.

## Capture summary

| Field | Value |
|-------|-------|
| Oracle file | `packages/codec/vectors/v4-codec.json` |
| Source | vl/app TS codec at `ignromanov/voidpay` master `c658fff` (release v1.1.2) |
| Vectors verified | 21/21 non-malformed vectors byte-identical (round-trip: canonical_hex + wire_hex) |
| Proof script | `packages/codec/scripts/vlapp-provenance.test.ts` |
| Proof run on codec commit | `285dd4b` |
| Proof run date | 2026-05-30 |
| Related decision | `2026-05-29-codec-d1-frozen-vectors-oracle` |

## Decode-only fixtures

The 2 `decode_unknown_odd_tag_*` vectors (`roundtrip: false`) are **decode-only
forward-compat fixtures**. They are excluded from encoder provenance because the
encoder never emits them — their canonical_hex was hand-crafted to exercise the
odd-ignore rule (BOLT-12). They are included in the `content_hash` so any edit
is still detected.

## Re-capture procedure

Re-capture is required when the vl/app TS codec changes in a way that alters
encoded output (schema change, compression strategy change, etc.).

1. Checkout vl/app at the new SHA.
2. Run: `VOIDPAY_SRC=/path/to/voidpay/src pnpm exec vitest run scripts/vlapp-provenance.test.ts --config scripts/vlapp-provenance.config.ts`
3. If all vectors match: capture is confirmed unchanged (no re-stamp needed).
4. If vectors differ: re-generate with `pnpm check-vectors -- --write`, which
   re-stamps `content_hash`.
5. Update this file: new vl/app SHA, new codec commit, new date.
6. Obtain Kai review before merging.
