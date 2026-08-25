# Security policy

## Supported versions

| Version | Supported |
| --- | --- |
| 3.x | Yes |
| 2.x and earlier | No |

Only the latest v3 release receives security fixes.

## Reporting a vulnerability

Use GitHub private vulnerability reporting for this repository. If that feature is unavailable, email `dat.nguyen@bitbytelab.io` with the subject `Burp MCP security report`.

Include:

- affected version and commit;
- deployment context and Burp edition;
- reproduction steps or a minimal proof of concept;
- impact and required attacker access;
- suggested remediation, if known.

Do not include live credentials, private target data, customer traffic, or destructive payloads. You should receive an acknowledgement within 7 days. Remediation timing depends on severity and the affected release surface.

## Security model

Burp MCP exposes dual-use Burp Suite capabilities. The Kotlin extension defaults to IPv4 loopback plaintext, making the local machine the trust boundary. Remote binding is an explicit settings-panel option and requires mutual TLS: the server validates a client certificate and the Rust client validates the server certificate hostname/IP against the generated private CA.

The extension generates `ca.crt`, server identity, and client identity under `~/.config/burp-mcp/tls` by default. Keep `server.key` on the Burp host; copy only `ca.crt`, `client.crt`, and `client.key` to an authorized agent host. Private keys are credentials. Rotation replaces the CA and both identities, immediately revoking old copied bundles. Plaintext non-loopback endpoints are rejected by both Kotlin and Rust.

## Repository security controls

This public repository uses the controls available without a paid GitHub plan:

- `main` requires a pull request, one approving review, code-owner approval, approval after the latest push, resolved review conversations, an up-to-date branch, linear history, and the required CI/dependency-review checks;
- force pushes and branch deletion are disabled; the repository owner retains the administrator bypass because GitHub does not allow an author to approve their own pull request and this repository currently has one maintainer;
- Dependabot alerts and security updates, dependency review, CodeQL, OpenSSF Scorecard, secret scanning, push protection, and private vulnerability reporting are enabled;
- release assets carry checksums, a CycloneDX SBOM, and GitHub build-provenance attestations.

GitHub secret-scanning validity checks and non-provider pattern scanning are not enabled because those settings require organization-level or paid GitHub Secret Protection capabilities. The baseline public-repository secret scanning and push protection remain enabled.
