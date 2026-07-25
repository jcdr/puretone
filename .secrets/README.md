# Local secrets (never pushed to GitHub)

This directory holds **machine-local** signing material for Pure Tone.

Tracked in git: only this `README.md`.  
**Not** tracked: `signing.env`, `upload.keystore`, and any other files here.

## Create secrets once

From the repository root:

```bash
./scripts/init-local-secrets.sh
```

That script:

1. Uses **`apg`** to generate a strong password  
2. Writes `.secrets/signing.env` (mode 600)  
3. Creates `.secrets/upload.keystore` with `keytool`  

Back up the keystore and password offline (password manager + USB). Losing them blocks Play updates with the same upload key.

## Build for Play (APK + AAB)

```bash
./scripts/build-play-upload.sh
```

Outputs go to `out/` (also gitignored).

## Do not

- Commit or paste `signing.env` / keystore contents  
- Force-add ignored files under `.secrets/`  
- Share this folder in chat or tickets  
