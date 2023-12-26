//! Implementation for security checks on repositories

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use octocrab::{models::Repository, Octocrab};

mod default_worfklow_permissions;
pub use default_worfklow_permissions::*;

mod repository_secrets;
pub use repository_secrets::*;

use crate::Args;

/// Context for running a check against GitHub
pub struct CheckCtx<'c> {
    /// Arguments to the CLI
    pub args: &'c Args,
    /// GitHub API client
    pub gh: &'c Octocrab,
}

impl<'c> CheckCtx<'c> {
    pub fn new(args: &'c Args, gh: &'c Octocrab) -> Self {
        Self { args, gh }
    }
}

/// Represents the possible operations for a check
#[async_trait]
#[enum_dispatch]
pub trait Check {
    async fn run<'c>(&self, ctx: &'c CheckCtx<'c>, repository: &Repository) -> anyhow::Result<()>;
}

/// Represents all the available checks
#[enum_dispatch(Check)]
#[derive(Debug, Clone, strum::EnumIter, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum Checks {
    DefaultWorkflowPermissions,
    RepositorySecrets,
}
