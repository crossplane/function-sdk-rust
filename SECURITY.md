# Security Policy

This project is part of the [Crossplane][crossplane] CNCF project and follows
Crossplane's security process.

## Reporting a Vulnerability

To report a vulnerability, either:

1. Use GitHub's [private vulnerability reporting][ghsa]:
   - Navigate to the [Security tab](https://github.com/crossplane/function-sdk-rust/security)
     on this repository
   - Click "Advisories", then "Report a vulnerability"
   - Detail the issue, see below for information that helps triage it

2. Send an email to `crossplane-security@lists.cncf.io` detailing the issue.

You can typically expect a response within 24 hours acknowledging the issue
was received. If you don't hear back within 24 hours, reach out to a
[maintainer](OWNERS.md) directly to confirm receipt.

### Report content

Please include what you can of the following:

- The version of `function-sdk-rust` used, and any other relevant software
  versions (e.g. Crossplane, Rust)
- Detailed steps to reproduce the vulnerability
- Consequences of the vulnerability, and the severity you'd attribute to it

## Review process

Once a maintainer has confirmed a report, a draft security advisory will be
created on GitHub to discuss the issue with the reporter and, where relevant,
Crossplane's security advisors. If accepted, a timeline for a patch and public
disclosure will be agreed with the reporter.

## Public disclosure

Fixed vulnerabilities are disclosed via a GitHub [security advisory][ghsa] and
mentioned in the release notes of the fixed version.

## Supported versions

Only the latest published version of the `function-sdk-rust` crate receives
security fixes.

[crossplane]: https://www.crossplane.io
[ghsa]: https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing/privately-reporting-a-security-vulnerability
