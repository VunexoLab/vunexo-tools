---
status: locked
round: 1
---

# Vunexo Vault — V1 Product Spec

This is an AI context file. Before modifying product-facing code, read this document. Do not violate a locked decision below without explicitly proposing a change (an ADR under `.ai/decisions/`) first.

## Vision

Vunexo Vault — **the simplest open-source secret manager for local development.** A free, open-source, local-first, offline-first CLI tool for developers, built for the same "no cloud dependency, no account, runs entirely on your machine" ethos as its sibling apps, but for a different audience: Vunexo Billing and Vunexo Expense Manager are business-facing desktop GUI apps for small-business owners; Vunexo Vault is a developer-facing command-line tool. Different audience, different distribution model (a CLI binary, not a desktop installer) — same monorepo, same free/open-source/offline-first ethos.

**Product positioning: the simplest open-source secret manager for local development — not an enterprise/team secrets platform.** This distinction is load-bearing. Vunexo Vault deliberately does not compete with team/cloud platforms (Doppler, Infisical, 1Password) in V1.

**Guardrail to preserve through every later round**: the product's whole strength is its constraint — local secrets + a simple CLI + strong encrypted storage + protection against accidental git leaks. Later rounds (user flows, architecture, UX, crypto design) must not quietly widen this into a team/cloud/CI platform. If that's ever wanted, it is a deliberate, separate future-version decision (see "Explicitly out of scope" below) — never something Rounds 2–9 back into by accretion.

## In scope — V1

Five pillars:

1. **Encrypted local secrets vault**
2. **Environment-aware secrets**
3. **Run commands without `.env` files**
4. **Secret detection**
5. **Git pre-commit protection**

Locked V1 command surface (exact flags/output are a Round 5 decision; the command set itself is locked here):

```bash
vunexo init

vunexo secrets set API_KEY
vunexo secrets get API_KEY
vunexo secrets list
vunexo secrets remove API_KEY

vunexo env list
vunexo env use development

vunexo secrets run -- npm run dev

vunexo secrets scan
vunexo secrets scan --staged

vunexo hooks install
vunexo hooks uninstall
```

- `vunexo init` creates the local encrypted vault.
- `vunexo secrets set/get/list/remove` manage individual secret values.
- `vunexo env list/use` manage named environments (e.g. `development`/`testing`/`staging`/`production`, user-definable); secrets are scoped per environment.
- `vunexo secrets run -- <command>` decrypts the active environment's secrets and injects them as the **spawned child process's environment only** — it never writes a plaintext `.env` file.
- `vunexo secrets scan` / `scan --staged` scan a directory or the staged git diff for likely secrets using pattern + entropy heuristics.
- `vunexo hooks install/uninstall` manage a git pre-commit hook that runs the scanner against staged changes and blocks the commit if a likely secret is found.
- Cross-platform: Windows, macOS, Linux.

Mental model locked in the spec:

```text
                 Vunexo Vault
                      │
          ┌───────────┴───────────┐
          │                       │
     Secret Vault             Leak Scanner
          │                       │
     encrypted                  source
       storage                   scanning
          │                       │
          └───────────┬───────────┘
                      │
                 Git Protection
                      │
                 pre-commit
```

## Hard boundaries (locked)

1. **No network calls, no accounts, no server, no shared/team vault in V1** — everything stays on the local machine.
2. **Secret scanning is a heuristic safety net, not a guarantee.** False positives and false negatives are expected. It must never be marketed as guaranteed leak prevention, and it does not replace developer diligence or dedicated deep-scanning tools.
3. **Vunexo Vault never implements its own password-based cryptography.** It maps a human passphrase to an `age` identity/recipient using only `age`'s own established mechanisms and libraries. The exact mapping is a Round 6 decision, not designed here — but no custom KDF/cipher implementation is permitted regardless of what Round 6 picks.
4. **Vunexo Vault must never persist plaintext secret values to disk during normal operation.** The spawned child process's environment (for `secrets run`) is the one intentional, designed plaintext boundary; everything else — the vault file, any temp/cache paths — stays ciphertext-at-rest:

   ```text
                     Plaintext secret
                            │
                 ┌──────────┴──────────┐
                 │                     │
            encrypted vault       child process
                 │                     │
                disk               environment
                 │
             ciphertext
   ```

   Memory zeroization of secret values is a Round 6 consideration (best-effort where technically practical in Rust), not a guarantee.

## Explicitly out of scope — V1

CI/CD platform integrations (GitHub Actions, etc.), Docker integration, secret rotation, a pluggable/advanced detection engine beyond the V1 heuristic set, a team/shared vault, a self-hosted server, RBAC, audit logs, and any cross-machine vault sync.

No accounts, no cloud, no network calls of any kind — introducing any of these would change the product's category from a local developer tool to a hosted platform.

## Architecture principles frozen into the spec

- **First plain Rust binary crate in the repo.** `apps/vunexo-vault/` is a standalone binary crate (its own independent `Cargo.toml`) — unlike Vunexo Billing and Expense Manager, which are Tauri apps. **The CLI is, and remains, the primary interface** — the thing git hooks and automation actually call, and the one every other interface must stay behind.
- **Amendment (2026-09-06): an optional desktop companion GUI is permitted, on one condition.** The original "no GUI" boundary is relaxed only to the extent that `apps/vunexo-vault-gui/` (a Tauri+React app) may exist as a *second interface* to this exact crate — it is not permitted to introduce any new secret-handling code path. `apps/vunexo-vault/src/{domain,application,infrastructure}` are exposed via a `[lib]` target (`src/lib.rs`) specifically so the GUI can depend on the identical `age`-based crypto, vault format, and scanner the CLI uses, via a plain path dependency — never a reimplementation. This is a disclosed, deliberate amendment (per this file's own header instruction to propose changes explicitly rather than violate a locked decision silently), not a reversal of the CLI-first positioning: no team/cloud/network capability is introduced by the GUI's existence, and every hard boundary above still applies identically to both interfaces.
- **Encryption via the `age` crate.** No home-rolled cipher or KDF code anywhere in this project. Exact vault file layout/versioning is a Round 6 decision, not designed here (mirrors how Expense Manager deferred its money representation past Round 1).
- **Ciphertext-at-rest, always.** The vault must remain safe even if accidentally `git add`ed — its design does not rely on `.gitignore` as the only protection.
- **Cross-platform from day one** — same Windows/macOS/Linux target set as the sibling apps' CI matrix.
- **Independent from the two GUI apps** — no shared runtime, no shared data. Monorepo tooling conventions (pnpm workspace root, CI patterns, license) are reused only where they genuinely fit a CLI, not forced.

## License

MIT — reuses the repo-wide decision already recorded in `.ai/decisions/ADR-002-license.md`. No new ADR needed.

## V1 Definition of Done

V1 is complete only when a user can:

- Initialize a local encrypted vault.
- Set, get, list, and remove secrets, scoped to a named environment.
- Create/select named environments.
- Run a real command with the active environment's secrets injected, with no `.env` file ever touching disk.
- Scan a directory, or just the staged git diff, for likely secrets.
- Install a git pre-commit hook that blocks a commit containing a likely secret, and uninstall it.
- Do all of the above entirely offline, with zero network calls, on Windows, macOS, and Linux.

## Roadmap

1. Spec + foundation (this document, binary crate scaffold) — **done**
2. User flows (init → set secrets → choose environment → run → scan → install hook)
3. Storage schema (vault file layout on disk, per-environment structure, what's encrypted vs. metadata)
4. Application architecture (module layout: vault/crypto domain, CLI command layer, scanning engine, git-hook integration)
5. CLI/terminal UX (exact command surface, flags, exit codes, output format, error messages)
6. Crypto & scanning engine — pins exactly how a human passphrase maps to an `age` identity/recipient (established `age` mechanisms only, no custom KDF/cipher), the vault file format, the secret lifecycle (creation through zeroization-where-practical), and secret-detection pattern/entropy rules + test vectors. **Treated as a security-design gate**: Round 7 must not begin until all of the above is explicitly pinned — implementation choices must never be allowed to silently become the security design.
7. Implementation
8. Testing
9. Release (CI, packaging, docs)
