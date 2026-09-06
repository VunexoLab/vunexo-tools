---
status: locked
round: 6
---

# Vunexo Vault — Crypto & Scanning Engine

**This is the security-design gate.** Per `.ai/product-vunexo-vault.md`'s roadmap, Round 7 (implementation) may not begin until everything in this document is pinned. Nothing below should be treated as adjustable "implementation detail" during Round 7 — a change to any of it is a change to the security design and must come back through this document first.

## 1. Passphrase → encryption key: `age`'s own passphrase mechanism only

`age`'s file format has a native, spec-defined stanza for passphrase-based encryption (built on `scrypt` as the key-derivation function, with the scrypt work factor and a random salt embedded directly in the encrypted file itself — this is part of the age specification, not something Vunexo designs). Vunexo Vault uses this mechanism exclusively:

- **Encrypting** a vault: construct an `age` passphrase-based recipient from the user's passphrase, and encrypt the plaintext payload to it. The resulting `.age` file embeds the scrypt salt and work-factor parameters `age` itself chose — Vunexo Vault does not choose or configure KDF parameters.
- **Decrypting**: construct the matching passphrase-based identity from the same passphrase, and hand it to `age`'s decryptor. `age` reads the embedded salt/work-factor from the file itself.
- This satisfies the locked boundary from `product-vunexo-vault.md`: *"Vunexo Vault never implements its own password-based cryptography."* There is no Vunexo-authored KDF, no Vunexo-chosen cipher mode, no Vunexo-managed salt or nonce anywhere in this design — all of that is `age`'s own, independently-specified and audited mechanism.
- The specific `age`-ecosystem crate/version used is an implementation choice for Round 7, but it must be the actively-maintained Rust implementation of the `age` file format (not a partial reimplementation) — pinning the exact crate name/version happens in `Cargo.toml` at implementation time, not decided here.

## 2. Vault file contents and format

- Plaintext payload (before encryption): the flat TOML key→value map described in `storage-schema.md`.
- The entire payload is encrypted as one `age` message — see `storage-schema.md` for why (both keys and values are protected, not just values).
- No ASCII-armoring requirement — the `.age` binary format is fine since these files are never meant to be manually read or emailed; if `age`'s library defaults produce armored (text) output, that's acceptable too, it doesn't change the security properties.

## 3. Secret lifecycle in memory

1. Passphrase is read via a masked terminal prompt (or from `--passphrase-file`, per `cli-ux.md`) directly into a `secrecy::SecretString`-wrapped (or equivalent zeroize-on-drop) value — never into a plain `String` that could be casually `Debug`-printed or accidentally logged.
2. Decrypted secret values are likewise wrapped (`domain::vault::SecretValue`, per `application-architecture.md`) with redacted `Debug`/`Display` impls and best-effort zeroization on drop.
3. **"Best-effort" is the honest claim, not "guaranteed erased."** Rust's ownership model and a zeroize-on-drop wrapper prevent casual/accidental exposure (stray `println!("{:?}", ...)`, a panic handler dumping local state, a value living longer than intended) and stop the *common* mistakes. They do not defend against a sufficiently privileged attacker reading raw process memory, a core dump, or values the OS/allocator may have copied (e.g. into swap, or a moved-and-not-yet-overwritten buffer) before the zeroize runs. Documentation and any future security notes must state this precisely — never "secrets are wiped from memory," always "we zero what a normal code path controls."
4. The one designed plaintext exit point is `secrets run`'s child-process environment block (see `application-architecture.md`'s `ProcessRunner` port) — inherent to the feature's own purpose, not an oversight.

## 4. What this design explicitly does not attempt

- No hardware-backed key storage (OS keychain/TPM) in V1 — passphrase-only. A future round could add it as an *additional* option, never a replacement that weakens the passphrase path without the user's explicit choice.
- No protection against a compromised machine (keyloggers, memory-scraping malware, a malicious `age`-crate dependency) — out of scope for a local file-encryption tool, same as any comparable tool (`age`, `sops`, `pass`).
- No defense against a user choosing a weak passphrase — `age`'s scrypt work factor raises the cost of brute-forcing a weak passphrase, but does not make a weak passphrase strong. V1 does not enforce a minimum passphrase strength; a future round could add an optional strength check without this being a breaking format change.

## 5. Scanning engine

### Rule types

1. **Pattern rules** — regexes matching known secret *shapes*: AWS access key IDs (`AKIA[0-9A-Z]{16}`) and secret keys, GitHub tokens (`gh[pousr]_[A-Za-z0-9]{36,}`), Stripe keys (`sk_live_...`/`sk_test_...`), private-key PEM headers (`-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----`), JWT-shaped strings (three base64url segments joined by `.`), and generic `KEY|SECRET|PASSWORD|TOKEN\s*=\s*['"]?[A-Za-z0-9+/=_-]{12,}` assignments.
2. **Entropy rule** — a generic catch-all: any quoted or assigned string literal of length ≥ 20 whose Shannon entropy exceeds a fixed threshold (tuned during Round 7/8 against the test vectors below, not designed to a specific decimal here) is flagged as a possible secret even if no pattern rule matches it.

### Finding output

Never prints the full matched string — only a redacted preview (first 4 and last 4 characters, the middle replaced with `...`), per `application-architecture.md`'s `ScanFinding` type. The point of a leak scanner is to stop leaking secrets, including its own output.

### Test vectors (locked; Round 7's implementation must pass all of these)

**Must be flagged (true positives):**
- `AKIAIOSFODNN7EXAMPLE` (AWS access key ID shape)
- `ghp_16C7e42F292c6912E7710c838347Ae178B4a` (GitHub PAT shape)
- `sk_test_FAKEFAKEFAKEFAKE1234` (Stripe key shape — deliberately low-entropy/dictionary-like so it doesn't itself trip GitHub's own push-protection secret scanner, unlike an earlier version of this vector)
- `-----BEGIN RSA PRIVATE KEY-----` (PEM header, regardless of what follows)
- `eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dQw4w9WgXcQ` (JWT shape)
- `AWS_SECRET_ACCESS_KEY = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"` (generic assignment + high entropy)

**Must NOT be flagged (true negatives, to bound false positives):**
- `API_KEY = "your-api-key-here"` (placeholder text, low entropy, common in example `.env.example` files)
- `password = "changeme"` (common placeholder, low entropy)
- A normal sentence of prose, a UUID used as a non-secret identifier, and a base64-encoded small image data URI under the length threshold.

These vectors are the acceptance test for Round 8, not just illustrative — `application-architecture.md`'s verification guide names them as required integration tests.

## 6. Known, disclosed limitation of the scanning approach

Per `product-vunexo-vault.md`'s locked boundary, this is a heuristic safety net. It will miss cleverly obfuscated or split secrets, and it will occasionally flag legitimate high-entropy non-secrets (e.g. a hash, a compiled asset). Both are acceptable and expected — the tool's job is to catch the common, careless-mistake case, not to provide a formal guarantee.

## Round 6 definition of done

- The passphrase→key mechanism is pinned to `age`'s own native passphrase stanza, with an explicit statement of why this satisfies the "no custom KDF/cipher" boundary.
- The secret lifecycle's zeroization claim is written precisely enough that a future contributor cannot accidentally market it as stronger than it is.
- A concrete, testable set of true-positive/true-negative vectors exists for the scanning engine, ready to become Round 8's acceptance tests.
