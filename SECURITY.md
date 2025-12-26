# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.x     | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in ByteHub, please report it privately.

### How to Report

1. **Email:** Send details to [rajsarkarpc@gmail.com](mailto:rajsarkarpc@gmail.com)
2. **Subject:** Use prefix `[SECURITY] ByteHub: <brief description>`
3. **Include:**
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Any suggested fixes (optional)

### What to Expect

- **Response Time:** We aim to respond within 48 hours
- **Updates:** We'll keep you informed of our progress
- **Credit:** We'll credit you in our security advisories (unless you prefer anonymity)

### What NOT to Do

- ❌ Do not open public GitHub issues for security vulnerabilities
- ❌ Do not disclose the vulnerability publicly before we've addressed it
- ❌ Do not exploit the vulnerability beyond what's necessary to demonstrate it

### Scope

The following are in scope for security reports:

- Authentication/authorization bypasses
- SQL injection, XSS, CSRF vulnerabilities
- Signature verification bypasses (GitHub webhooks, Discord interactions)
- Data exposure or leakage
- Denial of service vulnerabilities

### Out of Scope

- Vulnerabilities in third-party dependencies (report to them directly)
- Social engineering attacks
- Physical security issues

## Security Best Practices

When deploying ByteHub:

1. **Environment Variables:** Never commit `.env` files
2. **Database:** Use SSL connections to your database
3. **Secrets:** Rotate webhook secrets and bot tokens periodically
4. **Updates:** Keep dependencies updated (we use Dependabot)

---

Thank you for helping keep ByteHub secure! 🔐
