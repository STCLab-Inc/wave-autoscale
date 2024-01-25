# CONTRIBUTING to Wave Autoscale

- [CONTRIBUTING to Wave Autoscale](#contributing-to-wave-autoscale)
  - [Prerequisites](#prerequisites)
    - [1. Install Dependencies](#1-install-dependencies)
    - [2. Prepare Environment](#2-prepare-environment)
    - [3. Run](#3-run)
  - [Coding Rules](#coding-rules)
    - [Rust](#rust)
      - [Naming](#naming)
  - [Commit Message Conventions](#commit-message-conventions)

## Prerequisites
We have tested on MacOS and Linux. We are not sure if it works on Windows.

### 1. Install Dependencies
- [Moonrepo >= 1.10.1](https://moonrepo.dev/docs/install)
- [Rust >= 1.70.0](https://www.rust-lang.org/tools/install)
- [Node.js >= 18.14.0, NPM >= 10.2.3](https://nodejs.org/en/download/)

If you are using Linux, you can install dependencies using the following commands.
```bash
apt-get update
apt-get install -y build-essential pkg-config libssl-dev
```

### 2. Prepare Environment
- Prepare Git Hooks
  ```bash
  $ moon sync hooks
  ```

### 3. Run
```bash
# Run Wave Autoscale in development mode
moon run controller:run
```

## Coding Rules

### Rust

#### Naming
- Use `shared_` prefix for shared code.
  ```rust
  type shared_hashmap = Arc<Mutex<HashMap<String, String>>>;
  ```

## Commit Message Conventions
- Use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) for commit messages.
- type(group name): subject
- List of group names
  - controller (core/wave-autoscale)
  - api-server (core/api-server)
  - data-layer (core/data-layer)
  - utils (core/utils)
  - web-app (web/web-app)
  - moon (.moon)
  - github (.github)
  - docs (docs)
  - deployment (deployment)
  - example (example)
- Example
  ```
  feat(controller): add new feature
  fix(api-server): fix bug
  docs(.github): update docs
  chore(example): update build scripts
  refactor(utils): refactor code
  test(data-layer): add test
  ```