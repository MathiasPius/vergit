use std::str::FromStr;

use anyhow::{anyhow, Context};
use clap::Clap;
use git2::{Cred, CredentialType, ObjectType, PushOptions, RemoteCallbacks, Repository};
use semver::Identifier;

#[derive(Clap)]
enum VersionField {
    Major,
    Minor,
    Patch,
    Prerelease,
}

impl Default for VersionField {
    fn default() -> Self {
        VersionField::Patch
    }
}

#[derive(Clap)]
struct BumpCommand {
    #[clap(arg_enum, about = "Defaults to 'prerelease' if current version is prerelease, otherwise 'patch'")]
    pub version_field: Option<VersionField>,
    #[clap(long, about = "Push the new tag to a remote repository immediately")]
    pub push: bool,
    #[clap(long, default_value = "origin", about = "Set the remote to push to")]
    pub remote: String,
    #[clap(long, about = "Create no tags, just print the updated tag")]
    pub dry_run: bool,
}

#[derive(Clap)]
enum Commands {
    Bump(BumpCommand),
}

#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    pub subcommand: Commands,
    #[clap(short, long, about = "Don't print the updated tag")]
    pub quiet: bool,
}

fn main() -> Result<(), anyhow::Error> {
    let opts = Opts::parse();

    let repository = Repository::open(std::env::current_dir()?)?;
    let latest_version = repository
        .tag_names(None)?
        .into_iter()
        .filter_map(Option::from)
        .map(semver::Version::from_str)
        .filter_map(Result::ok)
        .max()
        .unwrap();

    match &opts.subcommand {
        Commands::Bump(bump) => {
            let field_to_bump =
                bump.version_field
                    .as_ref()
                    .unwrap_or(if latest_version.is_prerelease() {
                        &VersionField::Prerelease
                    } else {
                        &VersionField::Patch
                    });

            let new_version = {
                let mut new_version = latest_version.clone();
                match field_to_bump {
                    VersionField::Major => {
                        new_version.increment_major();
                    }
                    VersionField::Minor => {
                        new_version.increment_minor();
                    }
                    VersionField::Patch => {
                        new_version.increment_patch();
                    }
                    VersionField::Prerelease => {
                        let identifier = new_version
                            .pre
                            .pop()
                            .with_context(|| anyhow!("no prerelease identifiers found"))?;

                        new_version.pre.push(match identifier {
                            Identifier::AlphaNumeric(identifier) => Err(anyhow!(
                                "latest version identifier is not purely numeric: {}",
                                identifier
                            )),
                            Identifier::Numeric(number) => Ok(Identifier::Numeric(number + 1)),
                        }?);
                    }
                }
                new_version
            };

            if !bump.dry_run {
                let signature = repository.signature()?;

                let tag = repository.find_tag(repository.tag(
                    &format!("{}", new_version),
                    &repository.head()?.peel(ObjectType::Commit)?,
                    &signature,
                    "",
                    false,
                )?)?;

                if bump.push {
                    let mut remote = repository.find_remote(&bump.remote)?;

                    let mut callbacks = RemoteCallbacks::new();
                    callbacks.credentials(|_, username, credential_type| {
                        let username = username.unwrap_or("git");

                        if credential_type.contains(CredentialType::USERNAME) {
                            return Cred::username(username);
                        }

                        Cred::ssh_key_from_agent(username)
                    });

                    let mut push_options = PushOptions::new();
                    push_options.remote_callbacks(callbacks);

                    remote.push(
                        &[&format!(
                            "refs/tags/{}",
                            tag.name()
                                .with_context(|| "tag was just created, but has no name?")?
                        )],
                        Some(&mut push_options),
                    )?;

                    remote.disconnect()?;
                }
            }

            if !opts.quiet {
                println!("{}", new_version);
            }
        }
    }

    Ok(())
}
