# Abscissa Component Research

## Overview

Abscissa is a security-oriented Rust application microframework that uses a component architecture (similar to ECS) for extensibility. Components are lifecycle-aware and use dependency injection.

## Component Trait

The `Component<A>` trait is the primary extension mechanism. Components must implement:

### Required Trait Bounds
- `Injectable<A>` where `A: Application`

### Lifecycle Hooks

1. **`after_config(&mut self, config: &A::Cfg) -> Result<(), FrameworkError>`**
   - Called when application configuration is loaded
   - Allows components to read configuration and initialize
   - Has mutable access to self

2. **`before_shutdown(&self, kind: Shutdown) -> Result<(), FrameworkError>`**
   - Called prior to application shutdown
   - Enables cleanup and resource release
   - Immutable access only

## Implementation Approaches

### Method 1: Derive Macro (Recommended)

```rust
#[derive(Component, Debug)]
#[component(core)]  // Optional: marks as core component
pub struct MyComponent {
    // Component state
}
```

This automatically implements the entire `Component` trait.

### Method 2: Manual Implementation

```rust
#[derive(Injectable, Debug)]
pub struct MyComponent {
    // Component state
}

impl<A: Application> Component<A> for MyComponent {
    fn after_config(&mut self, config: &A::Cfg) -> Result<(), FrameworkError> {
        // Custom initialization logic
        Ok(())
    }

    fn before_shutdown(&self, kind: Shutdown) -> Result<(), FrameworkError> {
        // Custom cleanup logic
        Ok(())
    }
}
```

## Component Registry

- Components are stored as boxed trait objects in a runtime registry
- Registry performs topological sort based on declared dependencies
- Startup order is automatically calculated

## Configuration Integration

Components access configuration through the `after_config` hook:
- Config is available as `&A::Cfg` reference
- Components can define traits that the app config must implement
- Configuration is TOML-based with serde parsing

## Example: Terminal Component

```rust
#[derive(Component, Debug)]
#[component(core)]
pub struct Terminal {}

impl Terminal {
    pub fn new(color_choice: ColorChoice) -> Self {
        super::init(color_choice);
        // Optional: install color-eyre for better error handling
        Terminal {}
    }
}
```

The Terminal component is a unit struct - the derive macro handles all lifecycle implementation.

## Key Takeaways for observlib-rs Integration

1. **Create a component struct** to wrap `OtelManager`
2. **Use `after_config`** to read OTLP endpoint and service name from config
3. **Use `before_shutdown`** to call `OtelManager::shutdown()`
4. **Store OtelManager** in component state for lifecycle management
5. **Implement configuration trait** for type-safe config access
6. **Use derive macro** for cleaner implementation

## References

- [Component Trait Documentation](https://docs.rs/abscissa_core/latest/abscissa_core/component/trait.Component.html)
- [Abscissa Core Documentation](https://docs.rs/abscissa_core/latest/abscissa_core/index.html)
- [GitHub Repository](https://github.com/iqlusioninc/abscissa)
- [Terminal Component Source](https://docs.rs/abscissa_core/latest/abscissa_core/terminal/component/struct.Terminal.html)
- [Introducing Abscissa Blog Post](https://iqlusion.blog/introducing-abscissa-rust-application-framework)
