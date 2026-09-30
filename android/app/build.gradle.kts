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

android {
    namespace = "com.jcdr.puretone"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.jcdr.puretone"
        minSdk = 26
        targetSdk = 36
        versionCode = 2
        versionName = "1.0.1"
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
