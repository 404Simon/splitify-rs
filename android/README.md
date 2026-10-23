# Splitify Android

Native Android client for Splitify (Kotlin + Jetpack Compose, Material 3). It talks
to the Splitify REST API under `/api/v1` using Bearer tokens.

The client is **offline-first**: reads come from a local Room snapshot cache, writes are
applied optimistically and queued in an outbox, and a sync engine reconciles with the
server when connectivity returns. See [`../docs/OFFLINE_FIRST.md`](../docs/OFFLINE_FIRST.md).

It covers login / register / groups, member lists, shared debts, transactions, balances,
shopping lists, invites, recurring debts, and places (map markers). Interactive MapLibre
map rendering is still pending.

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

Key libraries: Room `2.8.5` (KSP `2.3.12`), WorkManager `2.12.0`, Ktor `3.6.0`,
DataStore Preferences `1.2.1`.

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
5. **Offline check**: enable airplane mode, then create a debt, a transaction, a
   shopping item and a place; toggle an item. All changes must appear instantly and
   survive a force-stop/relaunch. Re-enable the network and wait for the sync banner to
   clear; verify the rows on the server/web.
6. **Offline cold start**: with cached data, force-stop while still offline and relaunch.
   The app must open on the groups screen (not the login screen) and show the cached
   data with the "Offline — showing saved data" banner.


## Architecture

```
app/src/main/java/com/splitify/app/
├── SplitifyApplication.kt      # creates AppContainer
├── MainActivity.kt             # setContent { SplitifyTheme { AppRoot() } }
├── di/AppContainer.kt          # manual DI: session, Room, sync engine, repositories
├── data/
│   ├── network/                # Ktor HttpClient, ApiException, SyncApi, AppJson
│   ├── dto/                    # @Serializable DTOs + group snapshot DTOs
│   ├── local/                  # Room database, outbox/mapping/snapshot DAOs
│   ├── cache/                  # LocalStore: snapshot cache + optimistic mutations
│   ├── sync/                   # SyncManager, connectivity, WorkManager worker
│   ├── domain/                 # BalancesCalculator (offline parity with server)
│   ├── auth/                   # TokenStore, SessionManager, AuthRepository
│   ├── groups/                 # GroupRepository
│   ├── debts/                  # DebtRepository
│   ├── transactions/           # TransactionRepository
│   ├── invites/                # InviteRepository (links + accept, online-only writes)
│   ├── shoppinglists/          # ShoppingListRepository
│   ├── recurring/              # RecurringDebtRepository
│   └── maps/                   # MapMarkerRepository
└── ui/
    ├── theme/                  # Material 3 color scheme + typography
    ├── AppRoot.kt              # session gate (splash -> auth -> main) + SyncBanner
    ├── navigation/AppNavHost.kt
    ├── auth/                   # AuthViewModel, LoginScreen, RegisterScreen
    ├── groups/                 # GroupsViewModel/Screen, GroupDetailViewModel/Screen
    │   └── tabs/               # DebtsTab, TransactionsTab, BalancesTab, InvitesTab,
    │                           # ShoppingListsTab, RecurringDebtsTab, PlacesTab
    └── components/             # LoadingScreen, ErrorBanner, SyncBanner
```

### Offline-first data flow

The full design lives in [`../docs/OFFLINE_FIRST.md`](../docs/OFFLINE_FIRST.md). In short:

- **Reads**: every screen reads from `LocalStore`, a JSON snapshot cache in Room. The
  list/detail renders immediately on a cold start and while offline.
- **Writes**: mutations update the cached snapshot and append an outbox operation in the
  same Room transaction. The UI updates instantly; dialogs close without waiting.
- **Sync**: `SyncManager` pushes the outbox in order (resolving local temp ids), then pulls
  `GET /api/v1/groups/{id}/snapshot` for groups with no pending work. Pulls are skipped
  when a local mutation happened mid-flight.
- **Triggers**: app start, connectivity regained, after mutations (coalesced), manual
  "Sync now", and a 15-minute WorkManager job.
- **Retries**: every mutation carries an `Idempotency-Key`; the server stores successful
  responses for 7 days, so a retry after a lost response cannot duplicate a row.
- **Auth**: a cached token + cached user grants access offline. `/auth/me` is only a
  background validation; only a real `401` signs the user out. Bearer tokens slide
  forward on the server, so active clients do not expire.
- **Multi-account**: the cache and outbox are owned by a user id. Signing in as a
  different account clears them; signing back in as the same account keeps queued edits.

- **Tokens**: the bearer token is encrypted with an AES-256-GCM key held in the
  Android Keystore before being written to app-private Preferences DataStore; the
  key never leaves the device. Authorization headers are sanitized from Ktor logs.
  Backups (`allowBackup`) and data extraction are disabled.
- **Release hardening**: R8 minification + resource shrinking, HTTPS-only network
  security config, and no debug logging.
- **Errors**: non-2xx JSON `{"error": "..."}` responses become `ApiException.Http` with the
  server message; network and parse failures get friendly messages. Read errors are not
  per-screen banners: the global `SyncBanner` shows offline / syncing / pending / retry
  state instead.

## Endpoints used

Auth (`/auth/*`), groups (`/groups`, members), then per group: one
`GET /groups/{id}/snapshot` for debts, transactions, shopping lists (+ items, activity),
recurring debts (+ members, shares, instances), markers and invites; plus the mutation
endpoints for those resources. See `../docs/API.md` for the full list.

## Known limitations

- Balances are recomputed locally with `BigDecimal`; unit tests cover parity with the
  server formula, but there is no end-to-end UI test suite yet.
- Interactive MapLibre map rendering is deferred; the **Places** tab lists, adds, edits
  and deletes markers and can hand coordinates off to the system Maps app. Marker
  changes themselves work offline.
- Invite creation/revocation and accepting an invite are online-only (they need a
  server-issued link or membership change). Invite lists are cached.
- Conflict resolution is last-write-wins per record; there is no interactive merge UI.
- There is no realtime push/SSE on Android: other members' changes appear on the next
  sync (app start, tab refresh, connectivity change, or the 15-minute worker).
- Pull-to-refresh is not implemented; the global sync banner exposes a manual
  "Sync now" action and sync runs automatically.
- The Room cache is app-private and excluded from backups, but it is not separately
  encrypted (Android file-based encryption protects it at rest).
