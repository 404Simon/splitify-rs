import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.ksp)
}

// The API server address is not compiled in: the user enters it on the sign-in
// screen and the app remembers it (see `ServerConfig`). One APK therefore works
// against the emulator, a LAN host and production without a rebuild.

// Version: local builds use the defaults below, the release workflow passes the
// tag it is building, e.g. -PversionName=1.2.3 -PversionCode=10203.
val appVersionName: String = providers.gradleProperty("versionName").orNull ?: "0.1.0"
val appVersionCode: Int = providers.gradleProperty("versionCode").orNull?.toIntOrNull() ?: 1

// Release signing. GitHub Actions injects the keystore through environment
// variables; local builds read android/keystore.properties (both gitignored).
// Without either, `assembleRelease` still works but produces an unsigned APK.
val keystoreProperties = Properties().apply {
    val propertiesFile = rootProject.file("keystore.properties")
    if (propertiesFile.exists()) propertiesFile.inputStream().use { load(it) }
}

fun signingValue(environmentName: String, propertyName: String): String? =
    providers.environmentVariable(environmentName).orNull?.takeIf { it.isNotBlank() }
        ?: keystoreProperties.getProperty(propertyName)?.takeIf { it.isNotBlank() }

val releaseKeystorePath: String? = signingValue("SPLITIFY_KEYSTORE_PATH", "storeFile")
val releaseKeystorePassword: String? =
    signingValue("SPLITIFY_KEYSTORE_PASSWORD", "storePassword")
val releaseKeyAlias: String? = signingValue("SPLITIFY_KEY_ALIAS", "keyAlias")
val releaseKeyPassword: String? = signingValue("SPLITIFY_KEY_PASSWORD", "keyPassword")
val hasReleaseSigning: Boolean = releaseKeystorePath != null &&
    releaseKeystorePassword != null &&
    releaseKeyAlias != null &&
    releaseKeyPassword != null

// Personal-device builds: sign the minified release APK with the local debug
// key so it can update an already-installed debug build without uninstalling.
// Never enable this for a build you intend to distribute.
val useDebugSigning: Boolean =
    providers.gradleProperty("useDebugSigning").orNull?.toBoolean() == true

android {
    namespace = "com.splitify.app"
    compileSdk {
        version = release(37)
    }

    defaultConfig {
        applicationId = "com.splitify.app"
        minSdk {
            version = release(26)
        }
        targetSdk {
            version = release(36)
        }
        versionCode = appVersionCode
        versionName = appVersionName
    }

    signingConfigs {
        if (hasReleaseSigning) {
            create("release") {
                storeFile = rootProject.file(releaseKeystorePath!!)
                storePassword = releaseKeystorePassword
                keyAlias = releaseKeyAlias
                keyPassword = releaseKeyPassword
                // v1 signatures are only needed for API < 24; minSdk is 26.
                enableV1Signing = false
                enableV2Signing = true
            }
        }
    }

    buildTypes {
        debug {
            versionNameSuffix = "-debug"
        }
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
            when {
                hasReleaseSigning -> signingConfig = signingConfigs.getByName("release")
                useDebugSigning -> signingConfig = signingConfigs.getByName("debug")
            }
        }
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }

    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.androidx.lifecycle.viewmodel.compose)
    implementation(libs.androidx.activity.compose)

    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.material.icons.core)

    implementation(libs.androidx.navigation.compose)
    implementation(libs.androidx.datastore.preferences)

    implementation(libs.androidx.room.runtime)
    implementation(libs.androidx.room.ktx)
    ksp(libs.androidx.room.compiler)

    implementation(libs.androidx.work.runtime.ktx)

    implementation(libs.ktor.client.core)
    implementation(libs.ktor.client.okhttp)
    implementation(libs.ktor.client.content.negotiation)
    implementation(libs.ktor.serialization.kotlinx.json)
    implementation(libs.ktor.client.logging)

    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.kotlinx.serialization.json)

    testImplementation(libs.junit)
    testImplementation(libs.kotlinx.coroutines.test)

    debugImplementation(libs.androidx.compose.ui.tooling)
}
