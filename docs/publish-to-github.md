# Publish this tree to GitHub (`jcdr/puretone`)

## Before every push

Confirm secrets are not staged:

```bash
git status
# .secrets/signing.env and .secrets/upload.keystore must NEVER appear
git check-ignore -v .secrets/signing.env .secrets/upload.keystore
```

## Push

```bash
git remote add origin git@github.com:jcdr/puretone.git   # once
git push -u origin main
```

Use the GitHub noreply commit email if the remote rejects private addresses  
(`git config user.email "84882+jcdr@users.noreply.github.com"`).

## Privacy policy URL (optional Pages)

Repo → Settings → Pages → branch `main`, folder `/docs`.
