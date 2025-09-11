# Contribution guidelines

First off, thank you for considering contributing to ofxy!

If your contribution is not straightforward, please first discuss the change you wish to make by creating a new issue, ideally before investing your valuable time in making a change that may not be the right fit for this project.

## Reporting issues

Before reporting an issue on the [issue tracker](https://github.com/n8henrie/ofxy/issues), please check that it has not already been reported by searching for some related keywords.

## Pull requests

Please try to submit one pull request per logical change.
If the PR implements a new feature or behavior, tests and updates to documentation will generally be requested prior to merging.
If the PR fixes a bug in existing behavior, a separate commit with a *failing* test should first be submitted,  demonstraing the bug, with a subsequent commit that fixes the bug and protects against the buggy behavior returning thereafter.

### Updating the changelog

Update the changes you have made in
[CHANGELOG](https://github.com/n8henrie/ofxy/blob/main/CHANGELOG.md)
file under the **Unreleased** section.

Add the changes of your pull request to one of the following subsections,
depending on the types of changes defined by
[Keep a changelog](https://keepachangelog.com/en/1.0.0/):

- `Added` for new features.
- `Changed` for changes in existing functionality.
- `Deprecated` for soon-to-be removed features.
- `Removed` for now removed features.
- `Fixed` for any bug fixes.
- `Security` in case of vulnerabilities.

If the required subsection does not exist yet under **Unreleased**, create it!

## Developing

### Set up

This is no different than most other Rust projects:

```shell
git clone https://github.com/n8henrie/ofxy
cd ofxy
cargo test
```

### Useful Commands

- Build and run release version:

```shell
cargo build --release
```

- Run Clippy:

```shell
cargo clippy --all-targets --all-features --workspace
```

- Run all tests:

```shell
cargo test --all-features
```

- Check to see if there are code formatting issues

```shell
cargo fmt --all -- --check
```

- Format the code in the project

```shell
cargo fmt --all
```
