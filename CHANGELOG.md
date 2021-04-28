# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1](https://github.com/pedrodotmc/gitlab-rescue/releases/tag/0.1.1) - 2021-04-27
### Added
- `openssl` vendored dependency for compiling `musl`.

## [0.1.0](https://github.com/pedrodotmc/gitlab-rescue/releases/tag/0.1.0) - 2021-04-27
### Added
- `get` command for getting a variable from GitLab CI/CD.
- `dotenv` command for generating a dotenv file with GitLab CI/CD variables (only `posix` and `fish` shells supported).
- Unit and integration tests added.
- GitHub action workflows to generate binaries for linux-gnu, linux-musl and macos_x64
- README, CHANGELOG and LICENSE files.
