use std::str::FromStr;

use anyhow::{anyhow, Context};
use clap::Clap;
use git2::{
    Cred, CredentialType, DescribeFormatOptions, DescribeOptions, ObjectType, PushOptions,
    RemoteCallbacks, Repository,
};
use indoc::indoc;
use semver::{Identifier, Version};

#[derive(Clap)]
enum Component {
    Major,
    Minor,
    Patch,
    Prerelease,
}

impl Default for Component {
    fn default() -> Self {
        Component::Patch
    }
}

#[derive(Clap)]
struct BumpCommand {
    #[clap(
        arg_enum,
        about = "Defaults to 'prerelease' if version has prerelease, otherwise 'patch'",
        long_about = indoc! {"
            Defaults to 'prerelease' if current version contains a prerelease component,
            otherwise it will default to 'patch'. If prerelease is specified, but no 
            prerelease component is found, it will fail.

            Bumping prerelease tags only works if the last identifier of the prerelease
            component of the version string is numeric.
            
            For example, the following tags CANNOT be bumped using the prerelease command:
                0.0.1           No prerelease tag found
                0.0.1-beta      Last identifier of the prerelease component is not a number
                0.0.1-alpha1    Last identifier of the prerelease component is not a number
                0.0.1-beta.1.a  Last identifier of the prerelease component is not a number
            
            The following tags CAN be bumped using the prerelease command:
                0.0.1-beta.1    => 0.0.1-beta.2
                0.0.1-alpha.3   => 0.0.1-alpha.4
                0.0.1-test.b.2  => 0.0.1-test.b.3
        "}
    )]
    pub component: Option<Component>,

    #[clap(long, about = "Push the new tag to a remote repository immediately", long_about = indoc!{"
        The newly created tag will be pushed to a remote repository.

        The remote to push to can be overridden with --remote and defaults to 'origin'.
    "})]
    pub push: bool,

    #[clap(long, about = "Search all tags within the repository, not just the immediate history of this branch", long_about = indoc! {"
        Instead of walking backwards in the currently checked out history to find a tag 
        to increment, vergit will look at all tags in the entire repository and increment 
        the highest absolute version it can find.
    "})]
    pub global: bool,

    #[clap(long, default_value = "origin", about = "Set the remote to push to")]
    pub remote: String,

    #[clap(long, about = "Create no tags, just print the updated tag", long_about = indoc! {"
        In dry-run mode, no changes will be made to the git repository at all, the
        resulting new tag that would otherwise be created is just printed instead.

        For example, in a repository with only the tag 0.0.1 the following command:
            $ vergit bump patch
        
        Will yield the following output to stdout:
            0.0.2
        
        But make no modifications to the git repository
    "})]
    pub dry_run: bool,
}

#[derive(Clap)]
enum Commands {
    #[clap(
        about = "Bump the latest version tag of the git repository in the working directory",
        long_about = indoc! {"
            Takes the most recent tag (according to semantic-versioning ordering) of the
            current branch of the repository in the working directory and increases the
            <component> of the version tag by one.

            For example:
                Running the following command
                    $ vergit bump minor --global
            
                In a repository with the following tags:
                    hello-world
                    0.0.1-beta.3
                    0.3.4
                    1.8.5

                Will create a new tag 1.9.0 pointing at HEAD
        "}
    )]
    Bump(BumpCommand),
}

#[derive(Clap)]
#[clap(long_about = indoc! {"
    Command-line utility for quickly incrementing and pushing semantic-versioning
    tags in a git repository.

    By default vergit will go backwards in history from HEAD and find the latest
    tagged commit and increment that, unless the --global flag is specified.

    Examples:
        Increment the highest tag in the entire repository by one.
            $ vergit bump major --global

        Increment major version by 1, and don't print the new tag to stdout.
            $ vergit bump major --quiet

        Increment the minor version of the latest tag, and push the tag to origin
            $ vergit bump minor --push

        Increment the patch version of the latest tag, and push the tag to myremote
            $ vergit bump patch --push --remote=myremote

        Calculate the incremented tag and output it, but do not create the tag.
            $ vergit bump prerelease --dry-run
"})]
struct Opts {
    #[clap(subcommand)]
    pub subcommand: Commands,
    #[clap(short, long, about = "Don't print the updated tag")]
    pub quiet: bool,
}

fn main() -> Result<(), anyhow::Error> {
    let opts = Opts::parse();

    match &opts.subcommand {
        Commands::Bump(bump) => {
            let repository = Repository::open(std::env::current_dir()?)?;

            let latest_version = if bump.global {
                repository
                    .tag_names(None)?
                    .into_iter()
                    .filter_map(Option::from)
                    .map(semver::Version::from_str)
                    .filter_map(Result::ok)
                    .max()
            } else {

                let mut describe_options = DescribeOptions::new();
                describe_options.describe_tags();

                let mut describe_format = DescribeFormatOptions::new();
                describe_format.abbreviated_size(0);

                let tag_name = repository.describe(&describe_options)?.format(Some(&describe_format))?;
                Some(Version::from_str(&tag_name).with_context(|| anyhow!("The atest tag in the current branch does not conform to the semantic versioning spec: {}", tag_name))?)
            }
            .with_context(|| "No semantic versioning tags found")?;

            let field_to_bump =
                bump.component
                    .as_ref()
                    .unwrap_or(if latest_version.is_prerelease() {
                        &Component::Prerelease
                    } else {
                        &Component::Patch
                    });

            let new_version = {
                let mut new_version = latest_version;
                match field_to_bump {
                    Component::Major => {
                        new_version.increment_major();
                    }
                    Component::Minor => {
                        new_version.increment_minor();
                    }
                    Component::Patch => {
                        new_version.increment_patch();
                    }
                    Component::Prerelease => {
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
