//! The `repository_secrets` lists secrets that are defined in a repository.
//!
//! Currently, this check only lists secret names found in the repository being analyzed. Since
//! secrets often represent credentials (passwords, tokens, etc.) and are a high-value target, it
//! makes sense to know if a repository contains secrets.
//!
//! If a secret name matches the `--repository-secrets-warn-secret-names` regular expression, the
//! secret will be printed as a warning instead of an information message. This allows filtering
//! more critical secrets from less critical ones.
//!
//! When running with `--fix`, this check currently does not do anything.
//!
//! # Sources
//!
//! - [GitHub Docs](https://docs.github.com/en/rest/actions/secrets?apiVersion=2022-11-28#list-repository-secrets)
//! - [GitHub REST API](https://docs.github.com/en/rest/actions/secrets?apiVersion=2022-11-28#list-repository-secrets)

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use octocrab::models::Repository;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::{CheckCtx, RepositoryCheck};

#[derive(Debug, Serialize, Deserialize)]
struct SecretList {
    total_count: i32,
    secrets: Vec<Secret>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Secret {
    name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Implementation for the `repository_secrets` check
#[derive(Default, Debug, Clone, Copy)]
pub struct RepositorySecrets;

#[async_trait]
impl RepositoryCheck for RepositorySecrets {
    #[tracing::instrument(name = "repository_secrets", level = "info", skip_all)]
    async fn run<'c>(&self, ctx: &'c CheckCtx<'c>, repository: &Repository) -> anyhow::Result<()> {
        let secrets: SecretList = ctx
            .gh
            .get(
                format!(
                    "/repos/{}/{}/actions/secrets",
                    repository
                        .owner
                        .as_ref()
                        .ok_or_else(|| anyhow!("missing owner"))?
                        .login,
                    repository.name
                ),
                Option::<()>::None.as_ref(),
            )
            .await?;

        for secret in &secrets.secrets {
            if ctx
                .args
                .repository_secrets_warn_secret_names
                .is_match(&secret.name)
            {
                warn!(secret_name = secret.name, "found secret");
            } else {
                info!(secret_name = secret.name, "found secret");
            }
        }

        if !secrets.secrets.is_empty() && ctx.args.fix {
            info!("nothing to fix regarding secrets yet");
        }

        Ok(())
    }
}
