# Chepstow - Inference Token CLI

Get an access token for the Isambard (BriCS) inference service.

## Install

Download the binary for your platform from the [releases page](https://github.com/isambard-sc/chepstow/releases), make it executable and put it on your `PATH`. For example, on Linux x86_64:

```sh
curl -L -o chepstow https://github.com/isambard-sc/chepstow/releases/latest/download/chepstow-linux-x86_64
chmod +x chepstow
mv chepstow ~/.local/bin/
```

## Usage

After installing, first run the `auth` command, and follow the link in your browser to login.

```sh
chepstow auth             # print a login URL to open in your browser
```

Then run the `token` command to print an API token.

```sh
chepstow token            # print a valid access token
```
