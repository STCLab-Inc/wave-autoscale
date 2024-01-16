# CONTRIBUTING to Wave Autoscale

- [CONTRIBUTING to Wave Autoscale](#contributing-to-wave-autoscale)
  - [Prerequisites](#prerequisites)
    - [1. Install Dependencies](#1-install-dependencies)
    - [2. Prepare Environment](#2-prepare-environment)
    - [3. Run](#3-run)
  - [Coding Rules](#coding-rules)
    - [Rust](#rust)
      - [Naming](#naming)

## Prerequisites
### 1. Install Dependencies
**Workspace**
- Instasll [Moonrepo >= 1.10.1](https://moonrepo.dev/docs/install)

**Rust**
- Install [Rust >= 1.70.0](https://www.rust-lang.org/tools/install)

**Node.js**
- Install [Node.js >= 18.14.0](https://nodejs.org/en/download/)

### 2. Prepare Environment
- Prepare Git Hooks
  ```bash
  $ moon sync hooks
  ```

### 3. Run
```bash
# Run
moon run controller:run
```

## Coding Rules

### Rust

#### Naming
- Use `shared_` prefix for shared code.
  ```rust
  type shared_hashmap = Arc<Mutex<HashMap<String, String>>>;
  ```
