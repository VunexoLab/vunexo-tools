//! `ListEnvironments`/`UseEnvironment` use cases (`user-flows.md` §2).
//!
//! Neither touches `VaultStore` or prompts for a passphrase: `config.toml`
//! carries no secret material, and per the locked flow, `env use` on a name
//! that doesn't exist yet only needs to register it in the config — the
//! actual encrypted `.age` file for a brand-new environment is created
//! lazily by the first `secrets set` against it (see
//! `application::secrets::set_secret`'s `load_or_empty` helper).
//!
//! This resolves a real tension in the locked docs: `user-flows.md` §2 says
//! `env use` on a new name "creates ... a new empty encrypted vault file ...
//! using the same passphrase as the vault as a whole" but, in the same
//! sentence, "No passphrase prompt on its own." Those two clauses can't both
//! be true if `env use` itself has to produce ciphertext — encryption
//! requires *some* passphrase, and there's nowhere to get one without
//! prompting. Deferring file creation to the first `secrets set` for that
//! environment satisfies the actually-load-bearing half of the sentence (no
//! prompt on `env use`) and still ends up with an encrypted vault file "for"
//! that environment the moment it holds any secret. Note also that
//! `storage-schema.md`'s "same passphrase across environments" is a matter
//! of user discipline, not a technically enforced invariant: each `.age`
//! file is independently encrypted, and nothing in this design can validate
//! passphrase consistency across files without decrypting another
//! environment's vault for comparison, which the locked flow never asks for.

use crate::domain::config::VaultConfig;
use crate::domain::vault::Environment;

use super::ports::ConfigStore;
use super::VaultError;

/// `vunexo env list`.
pub fn list_environments(config_store: &dyn ConfigStore) -> Result<VaultConfig, VaultError> {
    config_store.load()
}

/// `vunexo env use NAME`. Creates `NAME` in `config.toml` if it isn't
/// already there, then makes it active.
pub fn use_environment(
    config_store: &dyn ConfigStore,
    environment: Environment,
) -> Result<(), VaultError> {
    let mut config = config_store.load()?;
    if !config.has_environment(&environment) {
        config.add_environment(&environment, crate::domain::config::now_rfc3339());
    }
    config.set_active(&environment);
    config_store.save(&config)
}
