# MyDNA Technical Architecture - Cloud Sync & Identity

## 1. Cloud Sync (Google Drive AppDataFolder)
MyDNA is a Serverless, P2P desktop application. It does not rely on a central database.
To provide cross-device synchronization and data safety, MyDNA uses Google Drive REST API.

- **Storage Location:** `appDataFolder`
- **Data Synced:**
  - `local_events.db`: The user's entire local analytics and SQLite database.
  - `identity.key`: The Ed25519 Private Key used for P2P networking and digital signature.

## 2. Compile-Time Secrets (Enterprise Security)
To communicate with Google APIs, MyDNA requires an OAuth2 `Client ID` and `Client Secret`. 
Since it is a desktop app, these "secrets" are embedded within the binary.

**Security Measure:** We use `env!()` macros and a `build.rs` script.
During development (`cargo build`), the compiler reads the `.env` file (which is git-ignored) and bakes the strings directly into the binary.
- This prevents secrets from leaking on GitHub.
- End-users can simply run the compiled `.exe` without needing a `.env` file, resulting in a seamless 1-click login experience.
- Satisfies "No Mocking" and "Fail-Fast" enterprise rules.

## 3. P2P Identity Integrity
- When a user logs in on a new machine, MyDNA downloads `identity.key` from Google Drive and injects it into the Windows Credential Manager.
- This ensures `1 User = 1 P2P Identity`, preserving the integrity of cross-verifications across the network.
