# Private release vault (secrets repository)

This guide sets up a **local-only secrets git repository** that:

- holds the Play **upload keystore** and passwords  
- **clones the public Pure Tone source** from GitHub  
- builds and signs release artifacts  
- never requires the public repository to know where the vault lives  

**Do not put the vault path, name, or passwords in the public repo, in chat, or in tickets.**

---

## Model

```text
YOU (offline)
  │
  ▼
LOCAL SECRETS GIT  ──clone/fetch──►  PUBLIC github.com/jcdr/puretone
  keystore                          (source only)
  passwords
  build/sign scripts
  │
  ▼
signed APK/AAB → upload in Play Console (manual is fine)
```

Dependency direction: **vault → public only**.

---

## One-time setup (you alone)

### 1. Create an empty vault somewhere only you know

Pick any private directory name and location. Example pattern (change it):

```bash
mkdir -p "$HOME/SOME_PRIVATE_PATH/puretone-release-vault"
cd "$HOME/SOME_PRIVATE_PATH/puretone-release-vault"
git init
```

Do **not** add a GitHub remote for this repo (or only use offline encrypted backup remotes you control).

### 2. Install templates from the public Pure Tone tree

From **inside the empty vault** directory, run the installer against a checkout of the public source (local path or after you clone puretone):

```bash
# PUBLIC_CHECKOUT = path to your puretone source tree (this project)
bash "$PUBLIC_CHECKOUT/docs/private-release-vault/install-into-this-repo.sh"
```

That copies templates into the current directory and makes scripts executable.

### 3. Edit fake values

Edit these files **in the vault** (not in the public repo):

| File | Purpose |
|---|---|
| `config.env` | Public source URL/tag, package id reminders |
| `signing.env` | Keystore passwords and alias (chmod 600) |

Replace every `CHANGE_ME` value.

### 4. Create the upload keystore

Still inside the vault:

```bash
./scripts/create-upload-keystore.sh
```

Store passwords in a password manager. Copy `upload.keystore` to encrypted offline backup.

### 5. Build Play-compatible APK + AAB (uses public source)

`cargo-apk` APKs are often **rejected** by Play (“cannot be analyzed using aapt”).  
Use the Gradle packaging script instead:

```bash
./scripts/fetch-public-source.sh
./scripts/build-play-upload.sh
```

Output under `out/`:

- `PureTone-play-….aab` — preferred for Play  
- `PureTone-play-….apk` — also Play-compatible  

(The older `build-signed-release-apk.sh` is only for sideload/debug of cargo-apk packages, not for Play upload.)

### 6. Upload to Play (e.g. Internal app sharing)

1. Play Console → Pure Tone  
2. Internal app sharing (or closed testing release)  
3. Upload the **`.aab`** (or the new Gradle **`.apk`**) from `out/`  
4. Open the share link on the phone

### 7. Commit the vault locally

```bash
git add -A
git status   # must NOT show real passwords if you use git-crypt; otherwise ensure remote is never public
git commit -m "Initialize Pure Tone release vault"
```

Prefer **not** committing plaintext passwords: keep `signing.env` untracked if you want (see `.gitignore` options in the template). The default template tracks `signing.env.example` only and ignores `signing.env`.

---

## Day-to-day release

```bash
cd /path/to/your/vault   # only you know this path
./scripts/fetch-public-source.sh
./scripts/build-signed-release-apk.sh
# upload out/*.apk (or future AAB) in Play Console
```

---

## What the public repo must never contain

- Real keystore files  
- Real passwords  
- Your vault path  
- Environment variable names that point to your vault  
- Scripts that load secrets from outside for “normal” app development  

Development of the app itself stays in the public tree with debug APK only.

---

## Recovery if the PC dies

1. Password manager → passwords  
2. Offline backup → `upload.keystore` (+ optional vault git bundle)  
3. GitHub → clone `jcdr/puretone`  
4. Recreate vault with `install-into-this-repo.sh`  
5. Restore keystore + `signing.env` → build again  

---

## Identity reminder (public product)

| Field | Value |
|---|---|
| App name | Pure Tone |
| Package | `com.jcdr.puretone` |
| Publisher | jcdr |
| Support email | puretone.support@gmail.com |
| Public source | https://github.com/jcdr/puretone |

---

## Files in this folder (public templates only)

| Path | Role |
|---|---|
| `README.md` | This guide |
| `install-into-this-repo.sh` | Run **from** an empty vault to install templates |
| `templates/` | Skeleton copied into the vault |

After install, work only inside the vault; do not copy filled secrets back into the public tree.
