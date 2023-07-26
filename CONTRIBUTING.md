# CONTRIBUTING to Wave Autoscale

## Preparation

### Install Dependencies
**Workspace**
- Instasll [Moonrepo >= 1.10.1](https://moonrepo.dev/docs/install)

**Rust**
- Install [Rust >= 1.70.0](https://www.rust-lang.org/tools/install)
- Install [cargo-watch >= 8.4.0](https://crates.io/crates/cargo-watch)]
- Install [cargo-audit >= 0.17.6](https://crates.io/crates/cargo-audit)
**Node.js**
- Install [Node.js >= 18.14.0](https://nodejs.org/en/download/)

### Prepare Environment
- Prepare Git Hooks
  ```bash
  $ moon sync hooks
  ```

## Development

### When you work on core/wave-autoscale
```bash
# Run
moon run controller:[test|run]

# If you need a situation that metrics are sent to the database and your plan responses to the metrics
# Edit core/wave-autoscale/tests/simulation.rs for your situation
moon run controller:test-simulation
```

## Coding Rules

### Rust

#### Naming
- Use `shared_` prefix for shared code.
  ```rust
  type shared_hashmap = Arc<Mutex<HashMap<String, String>>>;
  ```
