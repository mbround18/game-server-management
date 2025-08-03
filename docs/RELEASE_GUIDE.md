# Release Guide

## Overview
This repository uses automated releases triggered by pushes to the `main` branch. Each app in the `apps/` directory is released independently with its own semantic versioning.

## Release Process

### Automatic Release (Recommended)
1. **Merge to main**: Push or merge changes to the `main` branch
2. **Workflow triggers**: GitHub Actions automatically:
   - Detects all apps in `./apps/*/Cargo.toml`
   - Runs tests and builds
   - Creates Docker images for each app
   - Tags releases with app-specific prefixes (e.g., `enshrouded-v1.0.0`)

### Manual Release
If you need to trigger a release manually:
1. Push to `main` branch
2. Monitor the "Docker Release" workflow in GitHub Actions
3. Releases are tagged as `{app_name}-v{version}`

## Docker Images
- **Registry**: `mbround18/gsm-reference`
- **Tags**: Each app gets tagged with its semantic version
- **Target**: `gh-runtime`

## Versioning
- Uses semantic versioning (semver)
- Each app has independent versioning
- Prefixed with app name (e.g., `enshrouded-v1.2.3`)

## Requirements
- Changes must pass Rust tests
- Docker build must succeed
- Requires `DOCKER_TOKEN` and `GH_TOKEN` secrets

## Troubleshooting
- Check GitHub Actions logs for build failures
- Ensure `Cargo.toml` files in `apps/` are properly formatted
- Verify secrets are configured in repository settings