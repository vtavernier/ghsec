use std::str::FromStr;

use clap::Parser;
use futures_util::{stream::FuturesUnordered, StreamExt, TryStreamExt};
use octocrab::{models::Repository, Octocrab};
use tokio::pin;
use tracing::{info, level_filters::LevelFilter, debug};
use tracing_subscriber::{filter::Directive, EnvFilter};

mod args;
use args::Args;

mod checks;
use checks::{Check, CheckCtx};

#[tracing::instrument(name="repository", level="info", skip_all, fields(repository = repository.full_name.as_ref().unwrap()))]
async fn process_repo<'c>(ctx: &'c CheckCtx<'c>, repository: Repository) -> anyhow::Result<()> {
    for check in ctx.args.checks.clone().into_iter() {
        debug!(check = %check, "running check");
        check.run(ctx, &repository).await?;
    }

    Ok(())
}

#[tokio::main(worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    // Load variables from .env
    dotenv::dotenv().ok();

    // Load arguments
    let args = Args::parse();

    let filter = EnvFilter::builder()
        .with_default_directive(if args.debug {
            Directive::from_str("ghsec=debug").unwrap()
        } else {
            LevelFilter::INFO.into()
        })
        .from_env_lossy();

    let fmt = tracing_subscriber::fmt().with_env_filter(filter);
    if args.json {
        fmt.json().init()
    } else {
        fmt.compact().init();
    };

    // Create client
    let gh = Octocrab::builder()
        .personal_token(args.github_token.unsecure().to_string())
        .build()?;

    // Print authentication information
    let current_user = gh.current();
    info!("Logged in as {}", current_user.user().await?.login);

    // Get target repositories
    let repos = current_user
        .list_repos_for_authenticated_user()
        .type_("owner")
        .send()
        .await?
        .into_stream(&gh);
    pin!(repos);

    // Context for running checks
    let ctx = CheckCtx::new(&args, &gh);

    // Build a FuturesUnordered
    let mut tasks = FuturesUnordered::new();
    while let Some(target_repo) = repos.try_next().await? {
        tasks.push(process_repo(&ctx, target_repo));
    }

    // Poll it
    while tasks.next().await.is_some() {}

    Ok(())
}
