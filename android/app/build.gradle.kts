plugins {
    id("com.android.application")
}

val uploadStoreFile = System.getenv("PURETONE_UPLOAD_STORE_FILE")
val uploadStorePassword = System.getenv("PURETONE_UPLOAD_STORE_PASSWORD")
val uploadKeyAlias = System.getenv("PURETONE_UPLOAD_KEY_ALIAS")
val uploadKeyPassword = System.getenv("PURETONE_UPLOAD_KEY_PASSWORD")
val hasUploadSigning =
    !uploadStoreFile.isNullOrBlank() &&
        !uploadStorePassword.isNullOrBlank() &&
        !uploadKeyAlias.isNullOrBlank() &&
        !uploadKeyPassword.isNullOrBlank()

val playMaxVersionCode = 2_100_000_000L
val schemeVersionPattern = Regex("[1-9][0-9]{7,9}")

fun parseSchemeVersion(rawVersion: String, source: String): Int {
    val version = rawVersion.trim()
    if (!schemeVersionPattern.matches(version) || version.toLong() > playMaxVersionCode) {
        throw GradleException(
            "Invalid Pure Tone version '$rawVersion' from $source: expected yymmddnn " +
                "(8 to 10 digits, no leading zero, at most $playMaxVersionCode). See docs/VERSIONING.md.",
        )
    }
    return version.toInt()
}

val explicitVersionSource =
    providers.gradleProperty("puretoneVersion").orNull?.let { it to "-PpuretoneVersion" }
        ?: providers.environmentVariable("PURETONE_VERSION").orNull?.let { it to "PURETONE_VERSION" }
val explicitVersionCode = explicitVersionSource?.let { (value, source) -> parseSchemeVersion(value, source) }

val cargoPackageVersion =
    rootProject.file("../Cargo.toml").readLines()
        .dropWhile { it.trim() != "[package]" }
        .drop(1)
        .takeWhile { !it.trim().startsWith("[") }
        .firstOrNull { it.trim().startsWith("version") && it.contains("=") }
        ?.substringAfter("\"")
        ?.substringBefore("\"")
        ?: "0.0.0"
val cargoSchemeVersionCode =
    cargoPackageVersion.removePrefix("0.0.")
        .takeIf { cargoPackageVersion.startsWith("0.0.") && schemeVersionPattern.matches(it) && it.toLong() <= playMaxVersionCode }
        ?.toInt()

val resolvedVersionCode = explicitVersionCode ?: cargoSchemeVersionCode ?: 1
val resolvedVersionName = explicitVersionCode?.toString() ?: "${cargoSchemeVersionCode ?: cargoPackageVersion}-local"

gradle.taskGraph.whenReady {
    val releaseTaskNames = allTasks.filter { it.path.startsWith(":app:") && it.name.contains("Release") }.map { it.path }
    if (releaseTaskNames.isNotEmpty() && explicitVersionCode == null) {
        throw GradleException(
            "Release build needs an explicit version: pass -PpuretoneVersion=yymmddnn or set PURETONE_VERSION " +
                "(normally done by scripts/build-play-upload.sh). See docs/VERSIONING.md.",
        )
    }
}

tasks.register("printVersion") {
    val versionCodeText = resolvedVersionCode.toString()
    val versionNameText = resolvedVersionName
    val versionSourceText = explicitVersionSource?.second ?: "local fallback (Cargo.toml $cargoPackageVersion)"
    doLast {
        println("versionCode=$versionCodeText")
        println("versionName=$versionNameText")
        println("source=$versionSourceText")
    }
}

android {
    namespace = "com.jcdr.puretone"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.jcdr.puretone"
        minSdk = 26
        targetSdk = 36
        versionCode = resolvedVersionCode
        versionName = resolvedVersionName
        ndk {
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64")
        }
    }

    signingConfigs {
        if (hasUploadSigning) {
            create("upload") {
                storeFile = file(uploadStoreFile!!)
                storePassword = uploadStorePassword
                keyAlias = uploadKeyAlias
                keyPassword = uploadKeyPassword
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
            if (hasUploadSigning) {
                signingConfig = signingConfigs.getByName("upload")
            }
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
