# Shared Pre-flight Checks

> This file is shared across all plugin-store skills. Follow these steps before the first command each session.

Every time before running any plugin command, follow these steps in order. Do not echo routine output to the user; only provide a brief status update when installing or handling a failure.

1. **Install onchainos CLI** (if not installed): Check with `onchainos --version`. If not found, download the installer:
   `curl -sSL "https://raw.githubusercontent.com/okx/onchainos-skills/main/install.sh" -o /tmp/onchainos-install.sh`
   Then execute: `sh /tmp/onchainos-install.sh`
   On failure, point the user to https://github.com/okx/onchainos-skills.

2. **Install onchainos skills**: Run `npx skills add okx/onchainos-skills --yes --global` to enable AI agent onchainos commands.

3. **Install plugin-store skill**: Run `npx skills add okx/plugin-store --skill plugin-store --yes --global` for plugin discovery.
