# ⚡ ByteHub

> **GitHub → Governance → Discord**

A powerful bridge that connects your GitHub repositories to Discord, providing real-time notifications, project governance, and community engagement tools.

[![License](https://img.shields.io/badge/license-Source%20Available-blue.svg)](LICENSE.md)
[![Rust](https://img.shields.io/badge/rust-1.91%2B-orange.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/AriajSarkar/ByteHub/actions/workflows/ci.yml/badge.svg)](https://github.com/AriajSarkar/ByteHub/actions/workflows/ci.yml)

<a href="https://buymeacoffee.com/rajsarkar0f" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" height="40"></a>

---

## ✨ Features

- 🔔 **Real-time GitHub Notifications** - Issues, PRs, Releases, Workflow runs
- 🏛️ **Project Governance** - Approve/deny projects via Discord commands
- 📢 **Smart Announcements** - Auto-announce releases and bounty issues
- 🤖 **Bot Filtering** - Automatically filter out bot activity
- 🧵 **Forum Integration** - Create dedicated forum channels per project
- 🔐 **Secure** - Signature verification for GitHub webhooks and Discord interactions

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.91+
- [Convex](https://convex.dev/) account (free tier available)
- Node.js 18+ (for Convex functions)
- Discord Bot Token
- GitHub Webhook Secret

### Environment Variables

```bash
cp .env.example .env
```

Edit `.env` with your credentials:

```env
CONVEX_URL=https://your-project.convex.cloud
GITHUB_WEBHOOK_SECRET=your_secret
DISCORD_PUBLIC_KEY=your_key
DISCORD_BOT_TOKEN=your_token
DISCORD_APPLICATION_ID=your_app_id
```

### Setup Convex

```bash
pnpm install
npx convex dev --once --configure=new
```

### Run Locally

```bash
cargo run
```

### Run with Docker

```bash
docker build -t bytehub .
docker run -p 3000:3000 --env-file .env bytehub
```

---

## 📡 Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/` | Health check |
| `GET` | `/health` | JSON status |
| `POST` | `/webhooks/github` | GitHub webhook receiver |
| `POST` | `/webhooks/discord` | Discord interactions endpoint |

---

## 🛠️ Discord Commands

| Command | Description |
|---------|-------------|
| `/setup-server` | Initialize ByteHub channels in your server |
| `/approve <repo>` | Approve a project for tracking |
| `/deny <repo>` | Deny/remove a project |
| `/submit-project <repo>` | Submit a project for approval |
| `/list` | List all tracked projects |

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test discord_interactions
cargo test --test github_issue
```

---

## 🏗️ Project Structure

```
src/
├── discord/       # Discord client, commands, formatters
├── github/        # GitHub webhook handling, events
├── governance/    # Project approval, rules, whitelist
├── router/        # Event dispatching
└── storage/       # Database layer

tests/
├── discord/       # Discord interaction tests
├── github/        # GitHub webhook tests
└── common/        # Shared test utilities
```

---

## 📜 License

This project is licensed under the **ByteHub Source Available License**.

- ✅ Free for revenue under $10,000/year
- 💰 3% royalty for $10K-$100K revenue
- 💰 5% royalty for $100K+ revenue
- 📝 Attribution required

See [LICENSE.md](LICENSE.md) for full terms.

---

## 🤝 Contributing

We welcome contributions! Please read our [Contributing Guide](CONTRIBUTING.md) before submitting PRs.

> ⚠️ By contributing, you agree to our [Contributor License Agreement](LICENSE.md#5-contributor-license-agreement).

---

## 🔒 Security

Found a vulnerability?

See [SECURITY.md](SECURITY.md) for our security policy.

---

## 💖 Support

If ByteHub helps your project, consider supporting development:

<a href="https://buymeacoffee.com/rajsarkar0f" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" height="50"></a>

---

Made with ❤️ in India
