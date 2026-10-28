# Security Policy

## Supported versions

| Version | Supported |
| ------- | --------- |
| 0.1.x   | ✅        |
| < 0.1   | ❌        |

Security fixes land on `main` and ship in the next patch release of the latest minor version.

## Reporting a vulnerability

**Please do not open a public issue, discussion, or pull request for a security problem.** Report it privately instead:

1. **Preferred:** [open a private vulnerability report](https://github.com/Trooper-lab/neopress/security/advisories/new) on GitHub.
2. **Or email** [billy@trooper.es](mailto:billy@trooper.es) with the subject line `Neopress security`.

Please include:

- the Neopress version or commit you tested,
- steps to reproduce, or a proof of concept,
- what an attacker could do with it (the impact),
- any suggested fix, if you have one.

## What to expect

- We aim to acknowledge your report within **5 working days** and give a first assessment within **10 working days**.
- We will keep you updated while we work on a fix, release it in a patch version, and publish a GitHub Security Advisory.
- We credit reporters in the advisory unless you prefer to stay anonymous.
- Please keep the issue private until a fix is released, or for 90 days after your report, whichever comes first. If we need longer, we will tell you why.

## Scope

In scope:

- `neopress-server`: the public site, the admin portal (`/n-admin`), the REST APIs (`/wp-json/*`), authentication and sessions, file uploads, and the importers (JSON bundle and WXR).
- The WordPress migrator plugin (`neopress-migrator.php`).

Out of scope:

- Vulnerabilities in the WordPress site you migrate *from*.
- Denial of service through sheer request volume.
- Reports from automated scanners without a demonstrated impact.
- Actions that require an already-compromised administrator account, unless they let the attacker escape the admin's normal permissions (for example, code execution on the server).

## Running Neopress safely

- Set `NEOPRESS_ADMIN_PASSWORD`, or change the generated first-run password right after your first login.
- Serve Neopress behind HTTPS (for example, through a reverse proxy).
- Keep regular backups of your database and `uploads/` folder.
