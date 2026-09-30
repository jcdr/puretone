# Versioning

Pure Tone uses one number, called the *version* below, for every place that needs a
version: the Play upload file, the git tag, `versionName`, `versionCode`, the Play
release name and the Cargo package version.

## Format

The version is computed from the build date **in UTC**:

```
version = (year - 2000) * 1000000 + month * 10000 + day * 100 + nn
```

`nn` is a per-day counter, `00` for the first release of the day, then `01` … `99`.
From 2000 to 2099 this is simply the 8 digits `yymmddnn`:

| Date (UTC)           | nn | Version     |
|----------------------|----|-------------|
| 2026-09-30, 1st build | 00 | `26093000` |
| 2026-09-30, 2nd build | 01 | `26093001` |
| 2026-10-01, 1st build | 00 | `26100100` |

How the version appears in each place:

| Place                          | Value                                                     | Example                         |
|--------------------------------|-----------------------------------------------------------|---------------------------------|
| Android `versionCode`          | version as integer                                        | `26093000`                      |
| Android `versionName`          | version as string                                         | `"26093000"`                    |
| Play Console release name      | version (prefilled from `versionName`)                    | `26093000`                      |
| `Cargo.toml` package version   | `<version>.0.0`                                           | `26093000.0.0`                  |
| git tag (annotated)            | `v<version>`                                              | `v26093000`                     |
| Upload files                   | `out/PureTone-<version>.aab`, `out/PureTone-<version>.apk` | `out/PureTone-26093000.aab`     |
| Release notes (optional)       | `fastlane/metadata/android/en-GB/changelogs/<version>.txt` | `changelogs/26093000.txt`      |

## Why one number

Android requires an increasing integer (`versionCode`) and allows any string as
`versionName`. Using the same value for both, and for the tag and file names, removes
any mapping table: a file name, a Play Console entry, a crash report or a git tag
identifies the same build directly. The date shows the age of a build at a glance.

## Constraints

### Android versionCode

- A positive integer; Google Play accepts at most 2,100,000,000.
- Each upload must have a higher `versionCode` than the previous one, and a code
  cannot be reused, not even for internal testing.
- The size of the jump between two codes does not matter.
- Play blocks downgrades, so the move from code 2 (release 1.0.1) to `26xxxxxx` is
  permanent: codes below it can never be used again.
- The largest 8-digit value, `99123199`, is far below the Play limit.
- `nn` allows 100 releases per UTC day.

Sources:
<https://developer.android.com/studio/publish/versioning>,
<https://support.google.com/googleplay/android-developer/answer/9859152>,
<https://developer.android.com/google/play/publishing/multiple-apks>

### Per-ABI APKs

Pure Tone is uploaded as an Android App Bundle (AAB). Play generates the per-ABI APKs
from the bundle itself, and they all share one `versionCode`, so nothing extra is needed.

The documented recipes for uploading separate per-ABI APKs do not fit this scheme:
the Android example `abiCode * 1000 + versionCode` produces codes that collide with other
days' versions, and the F-Droid convention `100 * versionCode + N` exceeds the Play limit
(`2609300000 + N`). If per-ABI APKs are ever needed, use `versionCode * 10 + abiCode`
(maximum `991231999` up to 2099, still below the limit until the end of 2209).

Sources:
<https://developer.android.com/build/configure-apk-splits>,
<https://f-droid.org/docs/Build_Metadata_Reference/>

### Android versionName

A free-form string and the only version shown to users. Semantic Versioning is
recommended but not required.

Source: <https://developer.android.com/studio/publish/versioning>

### Play Console release name

A label visible only in Play Console. It is prefilled from `versionName` and does not
have to be unique. Release notes are limited to 500 characters per language.

Source: <https://support.google.com/googleplay/android-developer/answer/9859348>

### Cargo package version

Cargo requires a Semantic Versioning `major.minor.patch` version, so a bare `26093000`
is rejected. `26093000.0.0` is valid. A date form such as `26.09.30` is rejected because
SemVer forbids leading zeros in numeric components. Each component is a 64-bit unsigned
integer, so the size of `26093000` is not a problem.

Sources:
<https://doc.rust-lang.org/cargo/reference/manifest.html#the-version-field>,
<https://semver.org/>

### cargo-apk (debug builds)

`cargo apk` derives the APK `versionCode` from the Cargo version and requires major,
minor and patch to be at most 255 each; it also refuses a `version_code` override in
`Cargo.toml`. `26093000.0.0` therefore fails with cargo-apk. `scripts/build-debug-apk.sh`
builds the debug APK with a temporary Cargo version `0.0.0+<version>` and restores
`Cargo.toml` and `Cargo.lock` afterwards. Debug APKs are never uploaded to Play, so their
`versionCode` does not matter.

Source: `VersionCode::from_semver` and `ApkBuilder::from_subcommand` in
<https://github.com/rust-mobile/cargo-apk> (`ndk-build/src/cargo.rs`, `cargo-apk/src/apk.rs`)

### git tag with a `v` prefix

`git check-ref-format` accepts both `26093000` and `v26093000`. But a name made only of
hexadecimal digits also reads as an abbreviated commit ID, and git then warns
`refname '26093000' is ambiguous`. The `v` prefix avoids that. Tags are annotated.

Sources:
<https://git-scm.com/docs/git-check-ref-format>,
<https://git-scm.com/docs/gitrevisions>

### GitHub releases

A GitHub release is attached to a tag; the GitHub CLI examples use the `v` prefix
(`gh release create v1.2.3`).

Source: <https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository>

### F-Droid / fastlane changelogs

Changelog files are named after the literal `versionCode`:
`fastlane/metadata/android/en-GB/changelogs/26093000.txt`.

Source: <https://f-droid.org/docs/All_About_Descriptions_Graphics_and_Screenshots/>

### UTC date

The date is taken in UTC, so builds made on different machines or in different time
zones still produce increasing versions.

### Lifetime

The formula above extends the scheme past 2099 (proposed by Jean-Christian de Rivaz):
the century counter `c = (year - 2000) div 100` becomes leading digits in front of
`yymmddnn`, and `c = 0` (2000–2099) adds nothing. Computing the version arithmetically,
instead of concatenating strings, gives this automatically.

- The last possible version of 2099 is `99123199`; the first of 2100 is `100010100`,
  which is larger, so versions keep increasing.
- The prefix can grow while the value stays at most 2,100,000,000: with `c = 20`
  (years 4000–4099) the maximum is `2099123199`, which fits. From 4100 (`c = 21`) the
  first version `2100010100` exceeds the Play limit. The scheme is valid until the end
  of 4099.
- `100010100.0.0` is still valid SemVer (no leading zero), and tags stay `v<version>`;
  9- and 10-digit numbers are also valid hex, so the `v` prefix is still needed.
- Years 2000–2009 would give fewer than 8 digits (a leading zero year is dropped by the
  integer). This does not matter: the scheme started in 2026, and the tools accept 8 to
  10 digits.

### History

Before this scheme, release 1.0.0 (versionCode 1) and release 1.0.1 (versionCode 2) were
published on Play internal testing. They have no git tags; 1.0.1 is commit `ee04312`.

## Implementation

- `scripts/next-version.sh` prints the next free version. It computes today's UTC base
  with the formula above and sets `nn` to one more than the highest `nn` of today among
  the local tags and the tags on `origin` (`git ls-remote`; if `origin` is unreachable it
  warns and uses local tags only; `--local` skips the remote). It fails if `nn` would
  exceed 99, and refuses a version that is not greater than the highest existing
  `v<number>` tag (or than code 2 from before the scheme).
- `android/app/build.gradle.kts` reads the version from `-PpuretoneVersion=<version>` or
  the `PURETONE_VERSION` environment variable and uses it as `versionCode` and
  `versionName`. The value must be 8 to 10 digits without a leading zero and at most
  2,100,000,000, otherwise the build fails. Release tasks fail if no version is given.
  Other builds fall back to `Cargo.toml`: `<version>.0.0` gives code `<version>` and name
  `<version>-local`; anything else gives code 1 and name `<cargo version>-local`.
  `./gradlew :app:printVersion [-PpuretoneVersion=…]` shows the resolved values.
- `Cargo.toml` in `main` always holds the last released version. It stays `1.0.1` until
  the first release built with this scheme; `scripts/build-play-upload.sh` sets
  `<version>.0.0` (and updates `Cargo.lock`) at release time and commits it.
- `scripts/build-play-upload.sh` builds, signs, copies the outputs to
  `out/PureTone-<version>.aab/.apk`, commits `Release <version>` and creates the annotated
  tag `v<version>`, all locally. It never pushes. If the build fails it restores
  `Cargo.toml` and `Cargo.lock`.

## How to cut a release

1. Commit or stash all changes to tracked files; the script refuses a dirty tree.
2. Optional: write release notes (at most 500 characters) to a file.
3. Run the build. The version defaults to `scripts/next-version.sh`; an explicit version
   can be given as argument or in `PURETONE_VERSION`.

   ```bash
   ./scripts/build-play-upload.sh --notes-file notes.txt
   ```

4. Check the result:

   ```bash
   git show --stat HEAD
   git tag -n v<version>
   ls -l out/PureTone-<version>.*
   ```

5. Upload `out/PureTone-<version>.aab` in Play Console. Keep the prefilled release name
   (`<version>`) and paste the release notes.
6. Push the release commit and the tag:

   ```bash
   git push origin main v<version>
   ```

A version that was built but never uploaded is harmless: the next run takes the next
number, and gaps between versions do not matter.
