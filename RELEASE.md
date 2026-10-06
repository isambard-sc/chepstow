# Making a release

Releases are completely automated by GitHub Actions:
1. Make sure the [changelog](./CHANGELOG.md) is up to date.
   A release cannot be made if there is nothing in the changelog in the `Unreleased` section.
2. Go to the [Release](https://github.com/isambard-sc/chepstow/actions/workflows/release.yml) workflow page.
3. Click "Run workflow" in the top right.
4. Make sure the `main` branch is selected.
5. In the box below, type `patch`, `minor` or `major`, depending on the [SemVer level](https://semver.org) of release to make.
   E.G. for the first release, type `0.1.0`.
6. Press the "Run workflow" button.

This will kick off a series of chained workflows, culminating in a new release appearing on the [Releases page](https://github.com/isambard-sc/chepstow/releases).

The workflow commits the version bump and changelog to `main` and pushes a tag, using an SSH deploy key.
The repository needs a deploy key with write access, with its private half stored as the `DEPLOY_KEY` Actions secret.
