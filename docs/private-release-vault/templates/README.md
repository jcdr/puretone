# Pure Tone — private release vault

This repository is **local only**. It must never be pushed to a public host.

It builds **Pure Tone** by cloning the public source and signing with the
**upload** keystore stored here. Google Play App Signing holds the app signing key.

## Quick start (after install-into-this-repo.sh)

```bash
cp config.env.example config.env
cp signing.env.example signing.env
chmod 600 signing.env
# edit both files — replace CHANGE_ME

./scripts/create-upload-keystore.sh
./scripts/fetch-public-source.sh
./scripts/build-signed-release-apk.sh
```

Upload the file printed under `out/` in Google Play Console.

## Layout

| Path | Role |
|---|---|
| `upload.keystore` | Play upload key (gitignored) |
| `signing.env` | Passwords (gitignored) |
| `config.env` | Public git URL / refs |
| `scripts/` | Fetch, keystore, build |
| `work/` | Ephemeral public checkout (gitignored) |
| `out/` | Signed artifacts (gitignored) |

## Recovery

Password manager + offline keystore backup + public GitHub source are enough to rebuild this vault on a new machine.
