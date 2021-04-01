use std::str::FromStr;

use clap::Clap;
use git2::Repository;
use semver::Version;

#[derive(Clap)]
enum VersionField {
    Major,
    Minor,
    Patch,
}

impl Default for VersionField {
    fn default() -> Self {
        VersionField::Patch
    }
}

#[derive(Clap)]
struct BumpCommand {
    #[clap(subcommand)]
    version_field: VersionField,
}

#[derive(Clap)]
enum Commands {
    Bump(BumpCommand),
}

#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    pub subcommand: Commands,
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
            let new_version = {
                let mut new_version = latest_version.clone();
                match bump.version_field {
                    VersionField::Major => {
                        new_version.increment_major();
                    }
                    VersionField::Minor => {
                        new_version.increment_minor();
                    }
                    VersionField::Patch => {
                        new_version.increment_patch();
                    }
                }
                new_version
            };

            println!("bumping {:#?} => {:#?}", latest_version, new_version);
        }
    }

    Ok(())
}
