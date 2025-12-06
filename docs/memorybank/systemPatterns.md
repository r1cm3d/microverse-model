# System Patterns

## Architecture
The project is designed to be a Rust library (`microverse-model`) that exports core simulation functionalities.
- **Current State**: Initial scaffolding with `src/main.rs`.
- **Target Architecture**:
    - **Core Library (`lib.rs`)**: Contains the `Microverse` struct, `Config` struct, and core simulation logic.
    - **Binary/Examples**: `src/main.rs` or `examples/` directory for demonstrating usage.

## Design Patterns
- **Configuration Pattern**: Uses a `Config` struct to separate configuration from execution logic.
- **Simulation Loop**: The `run()` method encapsulates the simulation lifecycle.
- **Modularity**: Code should be organized into modules (e.g., `simulation`, `config`, `world`) to maintain separation of concerns.

## Code Organization (Planned)
```
src/
  lib.rs        # Library entry point
  simulation.rs # Simulation engine logic
  config.rs     # Configuration definitions
  main.rs       # Optional CLI entry point or simple runner
```

## Conventions
- Follow standard Rust naming conventions.
- Use `Result` for error handling.
- Documentation for all public APIs.
