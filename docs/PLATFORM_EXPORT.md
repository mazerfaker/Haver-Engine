# Platform export / Експорт

Windows: run CHECK_WINDOWS.bat then BUILD_WINDOWS.bat. In editor save project, choose Gameplay PRO > Export Windows project bundle, then copy dist/ForgePlayer.exe to the bundle. Launch with no arguments from bundle directory or pass an absolute path to project.forge.json. Bundle currently includes all assets, no installer.

Android: not implemented. Requires Android SDK/NDK, JDK, Rust targets, Bevy Android activity entry point, packaged assets instead of filesystem paths, touch and lifecycle tests, Gradle and signing. Current standalone binary is desktop only. Do not rename EXE to APK.

iOS: not implemented. Requires macOS/Xcode, Rust iOS target, native application entry point, bundle assets and signing/provisioning. Current editor remains desktop only.
