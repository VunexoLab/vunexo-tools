<p align="center">
  <img src="apps/vunexo-billing/src/assets/vunexo-billing-logo.png" alt="Vunexo Billing" width="200">
</p>

# Vunexo Tools

Monorepo for VunexoLabs' free, open-source, offline-first desktop tools.

- **Project #1: [Vunexo Billing](apps/vunexo-billing/)** — invoicing software for small businesses. See [.ai/product.md](.ai/product.md) for the locked product spec.
- **Project #2: [Vunexo Expense Manager](apps/expense-manager/)** — expense tracking for small businesses (not accounting). See [.ai/product-expense-manager.md](.ai/product-expense-manager.md) for the locked product spec.
- **Project #3: [Vunexo Vault](apps/vunexo-vault/)** — the simplest open-source secret manager for local development (encrypted local vault, run commands without `.env` files, git pre-commit leak scanning). See [.ai/product-vunexo-vault.md](.ai/product-vunexo-vault.md) for the locked product spec.

Vunexo Billing and Vunexo Expense Manager are independent of each other — each its own SQLite database, own business profile, no coupling between them. Vunexo Vault is a different kind of tool entirely — a developer-facing CLI, not a business-facing desktop app — and shares no data or runtime with either. See [.ai/decisions/](.ai/decisions/) for architecture decision records shared across the monorepo.

**[Download Vunexo Billing](https://github.com/vunexolabs/vunexo-tools/releases)** for Windows, macOS, or Linux. Vunexo Expense Manager and Vunexo Vault haven't been released yet — see their READMEs ([Expense Manager](apps/expense-manager/README.md), [Vault](apps/vunexo-vault/README.md)) for current status.

Licensed under [MIT](LICENSE); see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for the dependency license audit behind that choice.
