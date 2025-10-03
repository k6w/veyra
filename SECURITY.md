# Security Policy

## Supported Versions

We currently support the following versions with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take the security of Veyra seriously. If you discover a security vulnerability, please follow these steps:

### 1. **Do Not** Open a Public Issue

Please do not report security vulnerabilities through public GitHub issues, discussions, or pull requests.

### 2. Report Privately

Send an email to: **security@veyra-lang.org** (or create a private security advisory on GitHub)

Include the following information:
- **Description** of the vulnerability
- **Steps to reproduce** the issue
- **Potential impact** of the vulnerability
- **Affected versions** (if known)
- **Suggested fix** (if you have one)
- **Your contact information** for follow-up questions

### 3. Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Timeline**: Depends on severity
  - Critical: 1-7 days
  - High: 7-30 days
  - Medium: 30-90 days
  - Low: Next scheduled release

### 4. Disclosure Policy

- We will work with you to understand and validate the vulnerability
- Once a fix is ready, we will coordinate the disclosure timeline with you
- We aim to release security fixes as soon as possible
- You will be credited in the security advisory (unless you prefer to remain anonymous)

## Security Best Practices

When using Veyra:

1. **Keep Updated**: Always use the latest stable version
2. **Review Dependencies**: Regularly update dependencies using `cargo update`
3. **Audit Code**: Use `cargo audit` to check for known vulnerabilities
4. **Sandbox Untrusted Code**: Be cautious when running untrusted Veyra programs
5. **Report Issues**: If you find something suspicious, report it

## Security Features

Veyra includes several security features:

- **Memory Safety**: Garbage collection prevents memory leaks
- **Type Safety**: Runtime type checking prevents type confusion
- **Bounds Checking**: Array access is bounds-checked
- **Safe Defaults**: Secure configuration by default

## Security Updates

Security updates will be announced via:
- GitHub Security Advisories
- Release notes
- Project README
- Security mailing list (if established)

## Recognition

We appreciate the security research community's efforts. Security researchers who responsibly disclose vulnerabilities will be:

- Credited in the security advisory (unless anonymity is preferred)
- Mentioned in release notes
- Listed in our SECURITY_CONTRIBUTORS.md file (coming soon)

## Questions?

If you have questions about this security policy, please open a discussion on GitHub or contact the maintainers.

---

Thank you for helping keep Veyra and its users safe!
