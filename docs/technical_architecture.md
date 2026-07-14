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

## 4. P2P Network & Data Integrity (Phase 2 & 2.5)
To support serverless Job Matching, MyDNA implements a robust Peer-to-Peer network using `libp2p`.

- **Transport & Discovery**: Uses TCP, Yamux (Multiplexing), Noise (Encryption), and Kademlia DHT for Peer Discovery.
- **Job Matching (Gossipsub)**: Users broadcast their `MatchIntent` via pub/sub topics (`/mydna/recruitment/1.0.0` or `/mydna/freelance/1.0.0`).
- **Data Integrity Shield**: 
  - To prevent modified versions of MyDNA from entering the network and polluting data, we hash the `local_events.db` and sign it with the user's `Ed25519` key to generate an **Integrity Snapshot**.
  - This snapshot is stored on the Kademlia DHT with a 7-day TTL.
  - The "Master Release Hashes" are strictly maintained in `releases.json` which is **hardcoded at compile time** via the `include_str!()` macro. This makes it impossible for attackers to bypass checks by modifying local config files.
  - When two peers match, they cross-verify each other's snapshot. If a peer's App version is missing from `releases.json`, the connection is instantly rejected.
