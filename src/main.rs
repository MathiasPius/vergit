use clap::Clap;
use git2::Repository;

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

    let tag_names = repository.tag_names(None)?;

    match &opts.subcommand {
        Commands::Bump(bump) => match bump.version_field {
            VersionField::Major => {
                
            }
            VersionField::Minor => {

            }
            VersionField::Patch => {
                
            }
        },
    }

    Ok(())
}
