---
status: locked
round: 3
---

# Vunexo Vault — Storage Schema

## On-disk layout

```
.vunexo/
├── config.toml          # unencrypted metadata — no secret material, ever
└── vault/
    ├── development.age  # age-encrypted secrets for the "development" environment
    ├── staging.age
    └── production.age
```

One encrypted file per environment, not one shared file with per-environment sections — this keeps the blast radius of a single decrypt operation scoped to one environment, and matches the mental model of "environments are separate secret sets" from `user-flows.md`.

## `config.toml` (unencrypted)

```toml
format_version = 1
active_environment = "development"

[environments]
development = { created_at = "2026-09-06T00:00:00Z" }
staging     = { created_at = "2026-09-06T00:00:00Z" }
production  = { created_at = "2026-09-06T00:00:00Z" }
```

Holds only: the vault format version (for future migrations), the list of environment names and their creation timestamps, and which environment is currently active. **No secret keys, no secret values, no passphrase material, no salt.** This file is safe to read without ever prompting for a passphrase — `vunexo env list` relies on that.

## `vault/<environment>.age` (encrypted)

The **plaintext payload**, before encryption, is a simple flat map:

```toml
API_KEY = "sk_live_..."
DATABASE_URL = "postgres://..."
```

That plaintext is encrypted as a single `age` payload using a passphrase-derived recipient (mechanism pinned exactly in `crypto-and-scanning-engine.md`). Both keys and values are inside the ciphertext — an attacker with only the `.age` file learns nothing about what secrets exist, not even their names. This is why `secrets list` requires a passphrase (see `user-flows.md` §3): there is no metadata-only listing of secret names, by design.

All environments in one vault share the same passphrase — Vunexo Vault does not support per-environment passphrases in V1 (not in the locked command surface; would also complicate `secrets run` needing to prompt once per command, not once per secret).

## What's explicitly NOT here

- **No backup/restore/export commands** — not part of the locked V1 command surface (`.ai/product-vunexo-vault.md`). A user manages `.vunexo/` with their own file tools (or `git`, since it's safe to commit) if they want a backup.
- **No salt file separate from the `.age` payload** — `age`'s own file format embeds everything a decrypt needs (see Round 6); Vunexo Vault does not manage salts itself.
- **No cache/temp files** — secrets never get written to an intermediate path between "decrypt" and "use." Decrypt happens directly into memory.

## Migration strategy

`config.toml`'s `format_version` is the single migration trigger. Any future change to the vault file's internal plaintext shape (e.g. adding per-secret metadata like a comment or last-modified timestamp) bumps this integer; a mismatch between the binary's supported version and the file's version is a clear "run `vunexo` vX.Y to use this vault" error, never a silent best-effort read. No migration tooling is built until a version bump is actually needed — V1 ships at `format_version = 1` with no migration path required yet.

## Round 3 definition of done

- Every file Vunexo Vault reads or writes is named and its exact contents (encrypted vs. plaintext) are pinned.
- The "safe to `git add` the whole `.vunexo/` directory" claim from `product-vunexo-vault.md` is verified structurally: the only unencrypted file (`config.toml`) contains zero secret material.
- A concrete migration trigger (`format_version`) exists even though V1 needs no migration yet.
