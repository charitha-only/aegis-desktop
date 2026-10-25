# Create a source application

From the framework checkout:

```text
node aegis-cli/bin/aegis.js create ../ExampleApp
```

The target must be a new directory outside the framework checkout. Its name starts with a letter and uses at most 64 ASCII letters, digits, underscores or hyphens. Existing directories are refused and never merged. A failure during copying can leave a partial new directory; inspect it before removing or retrying.

The generated project includes the Rust core source, typed UI bridge, a small React example, CLI, configuration and lockfiles. It excludes runtime vaults/sandboxes, build outputs, node_modules and Git history. Linked/special source files, `.env` files and certificate/private-key-shaped files are rejected. This filename policy is not a secret scanner for source-code literals: review your source before distributing a template.

Inside the generated directory, use `npm run setup`, `npm run dev`, `npm run build` and `npm run bundle`. Rust and Node.js are prerequisites. The source checkout is self-contained, rather than depending on an unpublished package from a package registry. There is no automatic framework upgrade mechanism yet; application version in `aegis.config.json` is distinct from the bundled framework package version.

The small example checks core connectivity. It is not a password-manager or production application template. The existing framework README and demonstration UI are unchanged. The runtime still opens an external browser; native embedded windows remain an unresolved milestone. Linux/macOS portable directories are not signed native installers.

`node --test aegis-cli/test/create.test.js` verifies the public command, help/version output, no-overwrite behavior and credential-shaped file rejection. `node aegis-cli/bin/scaffold-smoke.js` creates an isolated project, installs locked UI dependencies and builds its UI, then removes only its own temporary fixture. Three-OS CI runs these checks. Generated-core build and real GUI/device acceptance are separate checks; UI compilation alone does not establish them.
