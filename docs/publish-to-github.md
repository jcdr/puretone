# Publish this tree to GitHub (`jcdr/puretone`)

The local branch **`main`** is a **clean single-commit** history (no spy-era commits).  
Full local history is kept on **`backup/pre-github-master`** only on this machine—do not push that branch.

## 1. Create the empty repo on GitHub

In a browser (logged in as **jcdr**):

1. Open https://github.com/new  
2. Repository name: **`puretone`**  
3. Owner: **jcdr**  
4. Public  
5. **Do not** add README, license, or .gitignore (this tree already has them)  
6. Create repository  

## 2. Allow this computer to push

If `ssh -T git@github.com` fails with *Permission denied*:

1. Show your public key: `cat ~/.ssh/id_ed25519.pub`  
2. GitHub → Settings → SSH and GPG keys → New SSH key → paste  
3. Retry: `ssh -T git@github.com` (expect a success greeting for **jcdr**)

## 3. Push from this project directory

```bash
cd /path/to/this/checkout
git remote remove origin 2>/dev/null || true
git remote add origin git@github.com:jcdr/puretone.git
git push -u origin main
```

After a successful push, the public URL is:

**https://github.com/jcdr/puretone**

## 4. Optional: GitHub Pages for the privacy policy

1. Repo → Settings → Pages  
2. Source: Deploy from branch **main**, folder **`/docs`**  
3. Privacy URL becomes something like:  
   `https://jcdr.github.io/puretone/privacy.html`  
   or raw markdown via the blob URL until Pages is configured.

Update the Play listing privacy field when the URL is live.

## 5. After GitHub exists — private vault

In your **private** vault (path only you know):

```bash
# set PUBLIC_GIT_URL=https://github.com/jcdr/puretone.git in config.env
./scripts/fetch-public-source.sh
./scripts/build-signed-release-apk.sh
```

## What was finished before this push

- Rebrand: Pure Tone / `com.jcdr.puretone` / jcdr  
- Support email documented: puretone.support@gmail.com  
- Secrets purged from the public tree  
- Privacy policy + vault bootstrap templates  
- Clean `main` history for first public upload  
