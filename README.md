# vergit

Vergit is a command-line utility for quickly incrementing and pushing semantic-versioning
tags in a git repository. It was created in order to relieve some of the pain
of working with strictly versioned terraform modules across git repositories.

## Installation
Assuming you have cargo installed, vergit can be installed by running
`cargo install vergit`

## Usage
```
Command-line utility for quickly incrementing and pushing semantic-versioning
tags in a git repository.

Examples:
    Increment major version by 1, and don't print the new tag to stdout.
        $ vergit bump major --quiet

    Increment the minor version of the latest tag, and push the tag to origin
        $ vergit bump minor --push

    Increment the patch version of the latest tag, and push the tag to myremote
        $ vergit bump patch --push --remote=myremote

    Calculate the incremented tag and output it, but do not create the tag.
        $ vergit bump prerelease --dry-run

USAGE:
    vergit [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help
            Prints help information

    -q, --quiet
            Don't print the updated tag

    -V, --version
            Prints version information


SUBCOMMANDS:
    bump    
            Takes the highest absolute tag (according to semantic-versioning ordering) of
            the repository in the working directory that abides by the semantic-versioning
            spec, and increases the <component> of the version tag by one.
            
            For example:
                Running the following command
                    $ vergit bump minor
            
                In a repository with the following tags:
                    hello-world
                    0.0.1-beta.3
                    0.3.4
                    1.8.5
            
                Will create a new tag 1.9.0 pointing at HEAD
    help    
            Prints this message or the help of the given subcommand(s)

```
