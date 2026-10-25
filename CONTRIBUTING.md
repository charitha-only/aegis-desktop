# 🤝 Contributing to Aegis Desktop

Thank you for your interest in contributing to **Aegis Desktop**! We welcome bug fixes, documentation improvements, security enhancements, and feature contributions from the community.

---

## 🚀 Getting Started

### Prerequisites
Make sure you have the following installed on your machine:
- **Rust & Cargo** (v1.75+ recommended): [Install Rust](https://www.rust-lang.org/tools/install)
- **Node.js** (v18.0+ recommended) & **npm**

### Fork & Clone
1. Fork the repository to your own GitHub account.
2. Clone your fork locally:
   ```bash
   git clone https://github.com/charitha-only/aegis-desktop.git
   cd aegis-desktop
   ```
3. Install dependencies:
   ```bash
   npm install
   cd src-ui && npm install && cd ..
   ```

---

## 💻 Development Workflow

1. **Create a branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Start the development environment:**
   ```bash
   npm run dev
   ```
   * Runs the Rust Security Core on `127.0.0.1:48291`
   * Starts the Vite frontend on `http://localhost:5173`

3. **Verify Security Tests:**
   Before submitting code, ensure that all capability and penetration tests pass:
   ```bash
   npm run test:security
   ```

---

## 📝 Pull Request Guidelines

- Ensure your code adheres to standard Rust formatting (`cargo fmt`) and TypeScript linting.
- If you introduce a new IPC command, update both:
  1. `src-core/src/ipc/mod.rs` (or respective handler)
  2. `src-ui/src/aegis-bridge.ts` (type-safe wrapper)
  3. `aegis.config.json` (capability permissions)
- Ensure no security capabilities are loosened or weakened without explicit documentation.
- Provide a clear, descriptive title and summary in your Pull Request.
