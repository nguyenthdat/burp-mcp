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

Burp MCP exposes dual-use Burp Suite capabilities. The Kotlin extension listens only on IPv4 loopback, and the local machine is the trust boundary. Users remain responsible for authorization, workstation access, MCP client configuration, and target scope.
