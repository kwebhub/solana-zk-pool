# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| `v0.1.x` | ✅ Active |
| `< v0.1` | ❌ Not supported |

## Reporting a Vulnerability

**DO NOT create a public issue** — this reveals the vulnerability before it's fixed.

### Option 1: GitHub Security Advisories (recommended)

1. Open https://github.com/kwebhub/solana-zk-pool/security/advisories
2. Click **"Report a vulnerability"**.
3. Fill in the form.

### Option 2: Email

Send to `security@example.com` (TODO: replace with real address):

- **Subject:** `[SECURITY] solana-zk-pool — <brief description>`
- **Body:**
  - Description of the vulnerability.
  - Steps to reproduce.
  - Impact (what can be broken).
  - Possible fix (if any).

### What NOT to do

- ❌ Do not create a **public issue**.
- ❌ Do not post on **Twitter/X**, **Discord**, **Telegram**.
- ❌ Do not exploit on **mainnet** (only devnet at this point).
- ❌ Do not attack **other users' deposits** — only your own.

## What We Promise

- **Response within 48 hours** (usually faster).
- **Severity assessment within 5 days**.
- **Fix within 30 days** for critical vulnerabilities.
- **Public credit** (if you want) in `CHANGELOG.md` after the fix.

## What Counts as a Vulnerability

### Critical

- **Double-spend:** withdrawing the same deposit twice.
- **Proof forgery:** withdrawing without knowing the secrets.
- **SOL theft from vault:** withdrawing to someone else's address.
- **Verifier bypass:** a proof that passes without verification.

### Medium

- **DoS:** taking down backend / prover / merkle.
- **Privacy leak:** deanonymization of a deposit.
- **Replay attack:** reusing a nullifier.
- **XSS:** stealing a deposit note from localStorage.

### Low

- **Metadata leak:** IP addresses, user-agent.
- **Rate limit bypass.**
- **Log leak:** private data in logs.

## What Does NOT Count as a Vulnerability

- ❌ **Trusted setup** — known limitation, not a bug (see [docs/threat-model.md](docs/threat-model.md)).
- ❌ **Upgrade authority** — known limitation.
- ❌ **Backend sees commitments/nullifiers** — by design.
- ❌ **localStorage** — known limitation.
- ❌ **Lack of HTTPS on devnet** — known dev environment limitation.

## Security Best Practices for Contributors

### Secrets

- ❌ **Never** commit:
  - `~/.config/solana/id.json` (Solana keypair).
  - `.env` (private keys, passwords).
  - SSH keys.
- ✅ Use `.env.example` as a template.
- ✅ Check before commit: `git diff --cached | grep -iE "secret|password|keypair|private"`.

### Dependencies

- **Dependabot** — automatic PRs for updates (weekly).
- **`cargo audit`** — checks Rust dependencies.
- **`pnpm audit`** — checks Node.js dependencies.
- **`trivy`** — scans Docker images.

### Code

- **`cargo clippy -- -D warnings`** — required.
- **All `unsafe`** — with comment and justification.
- **All `unwrap()` / `expect()`** — only in tests or where failure is **impossible**.
- **All `pub` functions** — with doc-comments.

### Program

- **All PDAs** — verify `bump` via `ctx.bumps`.
- **All `Signer`** — verify explicitly.
- **All CPI** — verify `program_id`.
- **All `require!`** — with a clear message.

## Known Limitations

See [docs/threat-model.md](docs/threat-model.md) — full list.

**Summary:**

1. **Trusted setup** — Sunspot uses standard setup without MPC.
2. **Upgrade authority** — single key, no multisig.
3. **XSS** — deposit notes in localStorage.
4. **Backend sees commitments/nullifiers** — but not the link between them.

## Bug Bounty

**Not launched yet.** Planned in the future (after audit and mainnet).

## Credits

Thanks to everyone who responsibly reports vulnerabilities:

<!-- List will be updated -->

---

**Last updated:** 2026-09-21
