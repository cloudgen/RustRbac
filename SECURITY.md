# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.1.1 (current) | Yes |
| 1.1.0 | Yes |
| 1.0.2 | Yes |
| 1.0.1 | Yes |
| 1.0.0 | Yes |
| Older releases | Best-effort; prefer upgrading to current |

## Reporting a Vulnerability

Please **do not** open a public issue for security-sensitive reports when a private channel is available.

**Maintainer contact (email):** `cloudgen.wong@gmail.com`

- Source of contact: product author in [`LICENSE.md`](./LICENSE.md).
- Prefer email for vulnerability details, reproduction steps, and impact.
- Do not include a live database password or a session token in a public channel.

## Security Design Principles (CIAO)

This project follows **[CIAO](https://github.com/cloudgen/ciao)** / **CIAO-Lite**.

| Letter | Principle | Security application |
|--------|-----------|----------------------|
| **C** | **Caution** | Unknown verbs and missing permissions fail closed. An unknown data scope shows no rows. |
| **I** | **Intentional** | The menu, the command, and the web page call the same permission check. |
| **A** | **Anti-fragile** | A password file that is not mode 0600 is ignored. A non-loopback bind needs an explicit allow. |
| **O** | **Over-protect** | The built-in administrator account and role stay in place. The program does not call sudo. |

The web cookie is HttpOnly and SameSite=Lax. A form post carries a CSRF field. `--password` is visible in the process list. The sign-in question and the menu mask that field. Change the seeded passwords after the first sign-in. The text menu does not open until a sign-in succeeds.
