# Rocket Blog & Worktime Tracker

A modern, fast, and feature-rich blog application and Progressive Web App (PWA) work time tracker built with **Rust** and the **Rocket** web framework.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Rust Version](https://img.shields.io/badge/rust-1.70+-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## 🏗️ Architecture

This application consists of **two separate binaries**:

- **Blog Binary** (`blog`) - Complete blog platform with content management, WYSIWYG editor, tags, comments, and RSS feed.
- **Work Time Tracker Binary** (`worktime`) - PWA for time tracking with role-based wages.

## ✨ Features

- 📝 **Blog Management** - Create, edit, delete, and publish blog posts.
- ✨ **WYSIWYG Markdown Editor** - Visual markdown editing with EasyMDE integration.
- 📄 **Post Excerpts** - Auto-generated excerpts with intelligent content extraction.
- 🔐 **Authentication System** - Secure login/logout with admin privileges.
- 💬 **Comment System** - Enable readers to comment on blog posts with moderation.
- 🏷️ **Tag System** - Organize posts with filterable tags.
- 📱 **Responsive Design** - Bootstrap-based UI that works on all devices.
- 📧 **RSS Feed Generation** - Complete RSS feed implementation.

## 🚀 Quick Start (NixOS)

This project has been ported to use native Nix packaging and NixOS modules.

For detailed deployment and development instructions using Nix, please refer to the **[NixOS Deployment Guide](README-NIX.md)**.

### Local Development (via Nix)

```bash
# Clone the repository
git clone https://github.com/xiro-codes/rocket_blog.git
cd rocket_blog

# Enter the reproducible Nix development shell
nix develop

# Setup database (requires postgres running locally)
just migrate

# Run the app locally
cargo run --bin blog
```

## 🛠️ Development

### Available Commands (via `just`)
```bash
just migrate           # Run database migrations
just gen-models        # Generate SeaORM models from database
just doc               # Generate Rust documentation
cargo test             # Run tests
cargo clippy           # Run linter
cargo fmt              # Format code
```

## 🏗️ Technology Stack
- **Backend**: Rust with Rocket web framework
- **Database**: PostgreSQL with SeaORM for type-safe queries
- **Frontend**: Server-side rendered HTML with **MiniJinja** templates
- **Styling**: Bootstrap 5 for responsive design
- **Build System**: Nix Flakes + Cargo

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](docs/CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
