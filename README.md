# Chepstow - Inference Token CLI

Get an access token for the Isambard (BriCS) inference service (LiteLLM).

## Install

Download a binary from the [releases page](https://github.com/isambard-sc/chepstow/releases), or build from source:

```sh
cargo install --git https://github.com/isambard-sc/chepstow
```

## Use

```sh
chepstow auth             # print a login URL to open in your browser, then wait for you to log in
chepstow token            # print a valid access token, refreshing it if needed
chepstow whoami           # show the token's claims and expiry
chepstow models           # list the models LiteLLM offers, to check it accepts the token
chepstow logout           # revoke the refresh token and delete the cache
```

`token` prints only the token on stdout. If it can't get a valid token, it exits non-zero and asks you to run `chepstow auth` on stderr.

Tokens are cached per environment in your cache directory (for example `~/.cache/chepstow/dev.json` on Linux), readable only by you.

### Environments and options

| `--env`       | issuer                                                | LiteLLM base URL                            |
|---------------|-------------------------------------------------------|---------------------------------------------|
| `dev` (default) | `https://keycloak-dev.isambard.ac.uk/realms/isambard` | `https://apps-dev.isambard.ac.uk/inference` |
| `prod`        | `https://keycloak.isambard.ac.uk/realms/isambard`     | `https://apps.isambard.ac.uk/inference`     |

Every option can also be set with an environment variable:

| flag          | env var              | default                      |
|---------------|----------------------|------------------------------|
| `--env`       | `CHEPSTOW_ENV`       | `dev`                        |
| `--issuer`    | `CHEPSTOW_ISSUER`    | from `--env`                 |
| `--client-id` | `CHEPSTOW_CLIENT_ID` | `chepstow`                   |
| `--base-url`  | `CHEPSTOW_BASE_URL`  | from `--env`                 |
| `--scope`     | `CHEPSTOW_SCOPE`     | `openid`                     |

Use `--scope "openid offline_access"` for a refresh token that outlives your Keycloak session, if the client allows it.

## Claude Code

In `~/.claude/settings.json`:

```json
{
  "apiKeyHelper": "chepstow token",
  "env": {
    "ANTHROPIC_BASE_URL": "https://apps-dev.isambard.ac.uk/inference",
    "CLAUDE_CODE_API_KEY_HELPER_TTL_MS": "240000"
  }
}
```

Set `CLAUDE_CODE_API_KEY_HELPER_TTL_MS` below the access-token lifetime, so Claude Code re-runs `chepstow token` before the token expires.
Don't use `ANTHROPIC_API_KEY`: Claude Code sends it as `x-api-key`, not as a `Bearer` token.

## opencode and OpenAI-style tools

These tools read a static key, for example:

```sh
export OPENAI_BASE_URL=https://apps-dev.isambard.ac.uk/inference/v1
export OPENAI_API_KEY=$(chepstow token)
```

or opencode's `{env:OPENAI_API_KEY}` / `{file:...}` in its config.
The token isn't refreshed while the tool runs, so it expires mid-session.
The fix is a longer access-token lifespan on the Keycloak client, which is a server-side decision.
