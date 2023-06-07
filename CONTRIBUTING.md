# CONTRIBUTING to Wave Autoscale

## Coding Rules

### Rust

#### Naming
- Use `shared_` prefix for shared code.
  ```rust
  type shared_hashmap = Arc<Mutex<HashMap<String, String>>>;
  ```
