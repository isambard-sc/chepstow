# Chepstow - Inference Token CLI

Get an access token for the Isambard (BriCS) inference service.

## Install

Download a binary from the [releases page](https://github.com/isambard-sc/chepstow/releases).

## Use

```sh
chepstow auth             # print a login URL to open in your browser
chepstow token            # print a valid access token
chepstow whoami           # show the token's claims and expiry
chepstow models           # list the models the inference service offers, to check it accepts the token
chepstow logout           # revoke the refresh token and delete the cache
```

chepstow uses the prod environment by default. Use `--env dev` or `CHEPSTOW_ENV=dev` for dev.

Tokens are cached per environment in your cache directory (for example `~/.cache/chepstow/prod.json` on Linux), readable only by you.

## opencode and OpenAI-style tools

These tools read a static key, for example:

```sh
export OPENAI_BASE_URL=https://apps.isambard.ac.uk/inference/v1
export OPENAI_API_KEY=$(chepstow token)
```

or opencode's `{env:OPENAI_API_KEY}` / `{file:...}` in its config.

## Claude Code

In `~/.claude/settings.json`:

```json
{
  "apiKeyHelper": "chepstow token",
  "env": {
    "ANTHROPIC_BASE_URL": "https://apps.isambard.ac.uk/inference",
    "CLAUDE_CODE_API_KEY_HELPER_TTL_MS": "240000"
  }
}
```

Set `CLAUDE_CODE_API_KEY_HELPER_TTL_MS` below the access-token lifetime, so Claude Code re-runs `chepstow token` before the token expires.
Don't use `ANTHROPIC_API_KEY`: Claude Code sends it as `x-api-key`, not as a `Bearer` token.
