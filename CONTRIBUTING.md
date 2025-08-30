# Contributing

This is currently a solo project and will be for the foreseeable future, so this document (for now) is more a **NOTES TO SELF** than a guide for contributors.

Contributions are still welcome, though, this is just me trying to remember my own process.

The reason I created this is because I can spend weeks without touching this code, and I have to remember the process:

## Roadmap

The planned features for each version is documented in [ROADMAP.md](ROADMAP.md).

## Starting a new version

1. After merging a version, create a new branch from `main` named `vMAJOR.MINOR.PATCH`
2. In the new branch, run the script `./bump_version.py`. This will:
   1. Update the version in `Cargo.toml`
   2. Update the version in `package.json`
   3. Update the version in `README.md`
   4. Update the version in `pyproject.toml`

3. Create a draft PR
4. Follow the steps described in `ROADMAP.md`
5. Merge PR. The CI should publish to `npm` and `PyPI`
