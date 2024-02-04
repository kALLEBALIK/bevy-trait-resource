# bevy-trait-resource
A way to get resources that implements a specific trait.

| Bevy Version | Crate Version |
|--------------|---------------|
| 0.12         | 0.3           |
| 0.11         | ---           |
| 0.10         | 0.1           |

## Note
This crate is highly experimental.

The code is derived and heavily relies on previous work done in [bevy-trait-query](https://github.com/JoJoJet/bevy-trait-query).

## Use

```rust
use bevy::prelude::*;
use bevy_trait_resource::{trait_resource, TraitResourceExt};

#[trait_resource]
pub trait IncrementTrait {
    fn value(&self) -> i32;
    fn increment(&mut self);
}

#[derive(Resource, Default)]
struct NumberValueResource {
    value: i32,
}

impl IncrementTrait for NumberValueResource {
    // Trait implementation...
}

pub fn increment_value_system(world: &mut World) {
    for res_opt in world.get_resources_trait_mut::<dyn IncrementTrait>() {
        if let Some(res) = res_opt {
            res.increment();
        }
    }
} 

struct SomePlugin;

impl Plugin for SomePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource_as::<dyn IncrementTrait, NumberValueResource>();
        app.add_systems(Update, increment_value_system);
    }
}

```
You can also register a resource to multiple traits.
```rust
impl Plugin for SomePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NumberValueResource>();

        app.register_resource_as::<dyn IncrementTrait, NumberValueResource>()
           .register_resource_as::<dyn SomeOtherTrait, NumberValueResource>();
    }
}
```
Unregistering.
```rust
app.unregister_resource_from_trait::<dyn IncrementTrait, NumberValueResource>();
```
