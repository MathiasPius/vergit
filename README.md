# vergit

Vergit is a command-line utility for quickly incrementing and pushing semantic-versioning
tags in a git repository. It was created in order to relieve some of the pain
of working with strictly versioned terraform modules across git repositories.

## Installation
Assuming you have cargo installed, vergit can be installed by running
`cargo install vergit`

## Usage
```
vergit-bump 
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

USAGE:
    vergit bump [FLAGS] [OPTIONS] [component]

ARGS:
    <component>
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
             [possible values: major, minor, patch, prerelease]

FLAGS:
        --dry-run
            In dry-run mode, no changes will be made to the git repository at all, the
            resulting new tag that would otherwise be created is just printed instead.
            
            For example, in a repository with only the tag 0.0.1 the following command:
                $ vergit bump patch --dry-run
            
            Will yield the following output to stdout:
                0.0.2
            
            But make no modifications to the git repository

        --global
            Instead of walking backwards in the currently checked out history to find a tag 
            to increment, vergit will look at all tags in the entire repository and increment 
            the highest absolute version it can find.

    -h, --help
            Prints help information

        --push
            The newly created tag will be pushed to a remote repository.
            
            The remote to push to can be overridden with --remote and defaults to 'origin'.

    -V, --version
            Prints version information


OPTIONS:
        --remote <remote>
            Set the remote to push to [default: origin]
```
