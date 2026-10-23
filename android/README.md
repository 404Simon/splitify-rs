# Splitify Android

Native Android client for Splitify (Kotlin + Jetpack Compose, Material 3). It talks
to the Splitify REST API under `/api/v1` using Bearer tokens.

This module implements **Phases 3 and 4** of the plan in `../docs/ANDROID_API_PLAN.md`:
project skeleton, theme, navigation, Ktor client, Keystore-backed session storage,
login / register / groups, and group tabs for members, shared debts, transactions,
balances, shopping lists, invites, recurring debts, and places (map markers).
Interactive MapLibre map rendering and pull-to-refresh are still pending.

## Requirements

| Tool | Version |
| --- | --- |
| JDK | 21 (AGP 9 does not support JDK 27) |
| Android SDK Platform | `platforms;android-37` |
| Android SDK Build-Tools | `build-tools;36.0.0` |
| `ANDROID_HOME` / `ANDROID_SDK_ROOT` | pointing at the SDK |
| Gradle | 9.8.2 (via wrapper) |

`compileSdk` is **37** because Compose BOM `2026.09.00` ships Compose `1.12.1`, whose
AARs declare `minCompileSdk=37`. `targetSdk` stays at **36** and `minSdk` at **26**.

Set JDK 21 for the build, e.g. with Nix:

```bash
nix shell nixpkgs#jdk21 nixpkgs#gradle --command bash -lc 'cd android && ./gradlew assembleDebug'
```

## SDK setup

Gradle picks up the SDK from `ANDROID_HOME` / `ANDROID_SDK_ROOT`.

**With Nix / home-manager:** the SDK is declared in
`~/nix-config/modules/android.nix` (`androidenv.composeAndroidPackages`, pinned to
`platforms;android-37.0` + `build-tools;36.0.0`/`37.0.0` + `platform-tools;37.0.1` +
`cmdline-tools;22.0`). It exports `ANDROID_HOME`/`ANDROID_SDK_ROOT` and puts
`adb`/`sdkmanager` on `PATH`; no `local.properties` or manual install is needed.

**Without Nix:** install the SDK packages once, then create `android/local.properties`
(gitignored):

```bash
yes | sdkmanager --licenses
sdkmanager "platform-tools" "platforms;android-37.0" "build-tools;36.0.0"
```

(Note: API 37's package id is `platforms;android-37.0`, not `android-37`.)

```
sdk.dir=/path/to/Android/Sdk
```

## API base URL

The base URL is compiled into `BuildConfig.API_BASE_URL`:

- Debug default: `http://10.0.2.2:3000/api/v1/` (the Android emulator reaches the
  host machine at `10.0.2.2`).
- Override per build, e.g. a physical device on your LAN:

```bash
./gradlew assembleDebug -PapiBaseUrl=http://192.168.1.20:3000/api/v1/
```

- Release builds **must** use HTTPS and pass the URL explicitly:

```bash
./gradlew assembleRelease -PapiBaseUrl=https://api.example.com/api/v1/
```

Cleartext HTTP is permitted in the **debug** variant for any host so the app can reach
a local `http://` API (`app/src/debug/res/xml/network_security_config.xml`); release
uses `app/src/main/res/xml/network_security_config.xml`, which rejects cleartext
everywhere. `ApiClient` also fails fast if a non-debug build is configured with a
non-HTTPS URL or the build placeholder.

## Build and run

```bash
# from android/
./gradlew assembleDebug          # build the debug APK
./gradlew installDebug           # install on a connected emulator/device
./gradlew lintDebug              # Android lint

# build with a custom API base URL
./gradlew assembleDebug -PapiBaseUrl=http://10.0.2.2:3000/api/v1/

# release build (R8 minification + resource shrinking, HTTPS required)
./gradlew assembleRelease -PapiBaseUrl=https://api.example.com/api/v1/
```

The debug APK is written to `app/build/outputs/apk/debug/app-debug.apk`; the
unsigned release APK to `app/build/outputs/apk/release/app-release-unsigned.apk`.

### Gradle wrapper

The wrapper (`gradlew`, `gradle/wrapper/`) targets Gradle 9.8.2. If it is missing,
regenerate it with any local Gradle install:

```bash
gradle wrapper --gradle-version 9.8.2 --distribution-type bin
```

## Testing against a local API

See also `../docs/ANDROID_API_PLAN.md` for the endpoint contract.

### 1. Run the backend

```bash
cd /home/simon/dev/splitify-rs
# .env already sets DATABASE_URL=sqlite:splitify.db; migrations run on startup
cargo run --features ssr
# server on http://127.0.0.1:3000
```

For a **physical device**, bind all interfaces so the phone can reach the host, and
find your LAN IP (`ip addr`) to use as the API base URL later:

```bash
LEPTOS_SITE_ADDR=0.0.0.0:3000 cargo run --features ssr
```

The Android app only needs the `/api/v1` endpoints; the web pages are not required.

### 2. Install the debug app

```bash
export JAVA_HOME=/path/to/jdk-21
export ANDROID_HOME=/path/to/Android/Sdk
cd android

# emulator or USB device connected via adb:
./gradlew installDebug

# physical device on the LAN (server bound to 0.0.0.0):
./gradlew installDebug -PapiBaseUrl=http://<YOUR-LAN-IP>:3000/api/v1/
```

- **Emulator**: the default `http://10.0.2.2:3000/api/v1/` already points at the
  host's loopback, so no flag is needed.
- **Physical device (recommended)**: use `adb reverse` so the phone reaches your
  machine's `127.0.0.1` over USB — no LAN IP or firewall setup needed:

  ```bash
  adb reverse tcp:3000 tcp:3000
  ./gradlew installDebug -PapiBaseUrl=http://127.0.0.1:3000/api/v1/
  ```

  `adb reverse` must be re-run after reconnecting the device or restarting adb.
- **Physical device on Wi-Fi**: bind the server to `0.0.0.0` (see step 1) and pass
  `-PapiBaseUrl=http://<LAN-IP>:3000/api/v1/`. Debug builds allow cleartext to any
  host so this works over plain HTTP; release builds are HTTPS-only.

The APK is also written to `app/build/outputs/apk/debug/app-debug.apk`
(`adb install -r <apk>` to sideload).

### Linux host prerequisites (adb access)

- `adb` needs udev rules to talk to the phone as a normal user: install `android-udev`,
  then `sudo udevadm control --reload-rules && sudo udevadm trigger`.
- The rules grant access to the `adbusers` group, and to the locally logged-in user via
  `uaccess`. Add yourself, then log out/in: `sudo gpasswd -a "$USER" adbusers`.
- Troubleshooting: `adb kill-server && adb start-server`, then `adb devices`. If the
  phone shows as `unauthorized`, accept the "Allow USB debugging?" dialog on the device.
- Wireless (Android 11+, no USB/udev needed): Developer options → Wireless debugging →
  *Pair device with pairing code*, then `adb pair <ip:port> <code>` and `adb connect <ip:port>`.

### 3. Manual test flow

1. Register `alice`, create a group, add a shared debt and a transaction, then open
   the **Balances** tab — verify the net amounts.
2. In **Invites**, create an invite and use **Share**/**Copy link** (creator only).
3. Log out, register `bob`, and use the groups-screen **Join with invite** action to
   paste the link or uuid and accept it; `bob` should now see the group.
4. As either user, exercise **Shopping lists** (add/toggle/delete items, activity),
   **Recurring debts** (create monthly, toggle, generate now, view instances), and
   **Places** (add a marker, edit/delete, open in Maps).


## Architecture

```
app/src/main/java/com/splitify/app/
├── SplitifyApplication.kt      # creates AppContainer
├── MainActivity.kt             # setContent { SplitifyTheme { AppRoot() } }
├── di/AppContainer.kt          # manual DI: TokenStore, ApiClient, repositories
├── data/
│   ├── network/                # Ktor HttpClient + ApiException/ApiResult
│   ├── dto/                    # @Serializable DTOs (snake_case via @SerialName)
│   ├── auth/                   # TokenStore (Keystore-encrypted) + AuthRepository
│   ├── groups/                 # GroupRepository
│   ├── debts/                  # DebtRepository
│   ├── transactions/           # TransactionRepository
│   ├── invites/                # InviteRepository (links + accept)
│   ├── shoppinglists/          # ShoppingListRepository
│   ├── recurring/              # RecurringDebtRepository
│   └── maps/                   # MapMarkerRepository
└── ui/
    ├── theme/                  # Material 3 color scheme + typography
    ├── AppRoot.kt              # session gate (splash -> auth -> main)
    ├── navigation/AppNavHost.kt
    ├── auth/                   # AuthViewModel, LoginScreen, RegisterScreen
    ├── groups/                 # GroupsViewModel/Screen, GroupDetailViewModel/Screen
    │   └── tabs/               # DebtsTab, TransactionsTab, BalancesTab, InvitesTab,
    │                           # ShoppingListsTab, RecurringDebtsTab, PlacesTab
    └── components/             # LoadingScreen, ErrorBanner
```

- **Session flow**: `AuthViewModel` observes the token in DataStore. On launch, a stored
  token is validated against `GET /auth/me`; a `401` clears the token. `AppRoot` shows a
  splash while checking and navigating reactively to the login or groups route.
- **Tokens**: the bearer token is encrypted with an AES-256-GCM key held in the
  Android Keystore before being written to app-private Preferences DataStore; the
  key never leaves the device. Authorization headers are sanitized from Ktor logs.
  Backups (`allowBackup`) and data extraction are disabled.
- **Release hardening**: R8 minification + resource shrinking, HTTPS-only network
  security config, and no debug logging.
- **Errors**: non-2xx JSON `{"error": "..."}` responses become `ApiException.Http` with the
  server message; network and parse failures get friendly messages.

## Endpoints used

Auth (`/auth/*`), groups (`/groups`, members), then per group: debts, transactions,
balances, invites, shopping lists (+ items, activity), recurring debts (+ toggle,
generate, instances, shares), and map markers. See `../docs/API.md` for the full list.

## Known limitations

- No automated tests yet (`testDebugUnitTest` / `assembleDebug` still need a device or
  the SDK installed locally).
- Interactive MapLibre map rendering is deferred; the **Places** tab lists, adds, edits
  and deletes markers and can hand coordinates off to the system Maps app.
- Pull-to-refresh is not implemented yet; tabs refresh after mutations and via retry.
