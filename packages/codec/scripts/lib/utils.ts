/**
 * Small utility helpers for vector generation scripts.
 */

import { COMPRESSED_FLAG } from './wire-codec.js'

export function toHex(bytes: Uint8Array): string {
  return Buffer.from(bytes).toString('hex')
}

export function isCompressed(hex: string): boolean {
  if (hex.length < 4) return false
  return (parseInt(hex.slice(2, 4), 16) & COMPRESSED_FLAG) !== 0
}
