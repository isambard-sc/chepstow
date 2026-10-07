# Making a release

Releases are automated by GitHub Actions. The [Release](https://github.com/isambard-sc/chepstow/actions/workflows/release.yml) workflow bumps the version, dates the changelog, commits both to `main`, pushes a tag, builds the binaries and publishes the release.

## One-time setup: deploy key

The workflow pushes to `main` with an SSH deploy key, because `main` requires pull requests and only deploy keys are on the ruleset's bypass list.
If the `DEPLOY_KEY` secret is missing, the push falls back to the Actions token and is rejected with "Changes must be made through a pull request".

1. Create a key pair on your machine. Leave the passphrase empty, because Actions can't type one:
   ```sh
   cd "$(mktemp -d)"
   ssh-keygen -t ed25519 -N "" -C "chepstow release workflow" -f deploy_key
   ```
   This makes `deploy_key` (private) and `deploy_key.pub` (public).
2. Add the **public** key to the repository:
   1. Go to **Settings → Deploy keys → Add deploy key**.
   2. **Title**: `Release workflow`.
   3. **Key**: paste the contents of `deploy_key.pub` (`pbcopy < deploy_key.pub` copies it on macOS).
   4. Tick **Allow write access**, then click **Add key**.
3. Add the **private** key as an Actions secret:
   1. Go to **Settings → Secrets and variables → Actions → New repository secret**.
   2. **Name**: `DEPLOY_KEY`.
   3. **Secret**: paste the whole contents of `deploy_key`, including the `-----BEGIN` and `-----END` lines (`pbcopy < deploy_key`).
   4. Click **Add secret**.
4. Delete both local files: `rm deploy_key deploy_key.pub`.
5. Make sure **Settings → Rules → Rulesets → (the `main` ruleset) → Bypass list** includes **Deploy keys**.

## Each release

1. Make sure the [changelog](./CHANGELOG.md) is up to date.
   A release can't be made if the `Unreleased` section is empty.
2. Go to the **Actions** tab and click **Release** in the left sidebar.
3. Click **Run workflow** (top right of the list of runs).
4. Make sure the `main` branch is selected.
5. In the version box, type `patch`, `minor` or `major`, depending on the [SemVer level](https://semver.org) of the release, or an exact version.
   For the first release, type `0.1.0`. Cargo.toml is already 0.1.0, so `major` would make 1.0.0.
6. Click the green **Run workflow** button.

The release appears on the [Releases page](https://github.com/isambard-sc/chepstow/releases) once the run finishes, after about 15 minutes.
Afterwards, run `git pull` on `main` to get the "Release X.Y.Z" commit.
