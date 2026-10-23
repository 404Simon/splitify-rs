# R8 / ProGuard rules for Splitify release builds.

# Keep the Kotlin metadata needed by kotlinx.serialization reflection.
-keepattributes *Annotation*, InnerClasses
-dontnote kotlinx.serialization.**

# kotlinx.serialization: keep generated serializers and their Companions.
-keepclassmembers class kotlinx.serialization.json.** {
    *** Companion;
}
-keepclasseswithmembers class kotlinx.serialization.json.** {
    kotlinx.serialization.KSerializer serializer(...);
}
-keepclassmembers class com.splitify.app.** {
    *** Companion;
}
-keepclasseswithmembers class com.splitify.app.** {
    kotlinx.serialization.KSerializer serializer(...);
}
-keep,includedescriptorclasses class com.splitify.app.**$$serializer { *; }

# OkHttp / Okio ship their own consumer rules; keep the annotations OkHttp reads
# at runtime if it is used reflectively.
-dontwarn org.conscrypt.**
-dontwarn org.bouncycastle.**
-dontwarn org.openjsse.**
