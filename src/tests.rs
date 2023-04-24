use super::*;

#[trait_resource]
pub trait IncrementTrait {
    fn value(&self) -> i32;
    fn increment(&mut self);
}

#[trait_resource]
pub trait IncrementStringTrait {
    fn string_value(&self) -> String;
    fn increment_string(&mut self);
}

#[derive(Resource, Default)]
struct NumberValueResource {
    value: i32,
}

#[derive(Resource)]
struct NumberValueResource2 {
    string_value: String,
    value: i32,
}

impl Default for NumberValueResource2 {
    fn default() -> Self {
        NumberValueResource2 {
            string_value: "0".to_owned(),
            value: 0,
        }
    }
}

impl IncrementTrait for NumberValueResource {
    fn value(&self) -> i32 {
        self.value
    }
    fn increment(&mut self) {
        self.value += 1;
    }
}

impl IncrementTrait for NumberValueResource2 {
    fn value(&self) -> i32 {
        self.value
    }
    fn increment(&mut self) {
        self.value += 1;
    }
}

impl IncrementStringTrait for NumberValueResource2 {
    fn string_value(&self) -> String {
        self.string_value.clone()
    }

    fn increment_string(&mut self) {
        let i: i32 = self.string_value.parse().unwrap();
        self.string_value = (i + 1).to_string();
    }
}

pub fn increment_value_test_system(world: &mut World) {
    for res_opt in world.get_resources_trait_mut::<dyn IncrementTrait>() {
        if let Some(res) = res_opt {
            res.increment();
        }
    }
}

pub fn increment_string_value_test_system(world: &mut World) {
    for res_opt in world.get_resources_trait_mut::<dyn IncrementStringTrait>() {
        if let Some(res) = res_opt {
            res.increment_string();
        }
    }
}

#[test]
fn increment() {
    let mut world = World::new();
    world.init_resource_as::<dyn IncrementTrait, NumberValueResource>();

    let mut schedule = Schedule::new();

    schedule.add_system(increment_value_test_system);

    schedule.run(&mut world);
    schedule.run(&mut world);

    assert_eq!(world.resource::<NumberValueResource>().value(), 2);
}

#[test]
fn increment_multiple() {
    let mut world = World::new();
    world.init_resource_as::<dyn IncrementTrait, NumberValueResource>();
    world.insert_resource_as::<dyn IncrementTrait, NumberValueResource2>(NumberValueResource2 {
        string_value: "0".to_owned(),
        value: 3,
    });

    let mut schedule = Schedule::new();

    schedule.add_system(increment_value_test_system);

    schedule.run(&mut world);
    schedule.run(&mut world);

    assert_eq!(world.resource::<NumberValueResource>().value(), 2);

    assert_eq!(world.resource::<NumberValueResource2>().value(), 5);
    assert_eq!(world.resource::<NumberValueResource2>().string_value, "0");
}


#[test]
fn getting_removed_resource_should_not_panic() {
    let mut world = World::new();
    world.init_resource_as::<dyn IncrementTrait, NumberValueResource>();
    world.insert_resource_as::<dyn IncrementTrait, NumberValueResource2>(NumberValueResource2 {
        string_value: "1".to_owned(),
        value: 3,
    });

    let mut schedule = Schedule::new();

    schedule.add_system(increment_value_test_system);

    schedule.run(&mut world);

    assert_eq!(world.resource::<NumberValueResource>().value(), 1);

    assert_eq!(world.resource::<NumberValueResource2>().value(), 4);
    assert_eq!(world.resource::<NumberValueResource2>().string_value, "1");


    world.remove_resource::<NumberValueResource2>();

    schedule.run(&mut world);

    assert_eq!(world.resource::<NumberValueResource>().value(), 2);
    assert!(world.get_resource::<NumberValueResource2>().is_none());
}


#[test]
#[should_panic(expected = "Trying to register a nonexistent resource")]
fn registering_non_existing_resource_should_panic() {
    let mut world = World::new();
    world.register_resource_as::<dyn IncrementTrait, NumberValueResource>();
}

#[test]
fn register_resource_trait() {
    let mut world = World::new();
    world.init_resource::<NumberValueResource>();
    world.register_resource_as::<dyn IncrementTrait, NumberValueResource>();

    let mut schedule = Schedule::new();

    schedule.add_system(increment_value_test_system);

    schedule.run(&mut world);
    assert_eq!(world.resource::<NumberValueResource>().value(), 1);

    schedule.run(&mut world);
    assert_eq!(world.resource::<NumberValueResource>().value(), 2);
}

#[test]
fn register_multiple_traits_on_resource() {
    let mut world = World::new();
    world.init_resource::<NumberValueResource2>();
    world.register_resource_as::<dyn IncrementTrait, NumberValueResource2>();
    world.register_resource_as::<dyn IncrementStringTrait, NumberValueResource2>();

    let mut schedule = Schedule::new();

    schedule.add_system(increment_value_test_system);
    schedule.add_system(increment_string_value_test_system);

    schedule.run(&mut world);
    assert_eq!(world.resource::<NumberValueResource2>().value(), 1);
    assert_eq!(world.resource::<NumberValueResource2>().string_value(), "1");

    schedule.run(&mut world);
    assert_eq!(world.resource::<NumberValueResource2>().value(), 2);
    assert_eq!(world.resource::<NumberValueResource2>().string_value(), "2");
}

#[test]
fn unregister_trait_from_resource() {
    let mut world = World::new();
    world.init_resource::<NumberValueResource>();
    world.init_resource::<NumberValueResource2>();

    world.register_resource_as::<dyn IncrementStringTrait, NumberValueResource2>();
    world.register_resource_as::<dyn IncrementTrait, NumberValueResource2>();
    world.register_resource_as::<dyn IncrementTrait, NumberValueResource>();

    match world.get_resource::<TraitResourceRegistry<dyn IncrementTrait>>() {
        Some(registry) => {
            assert_eq!(registry.trait_data.len(), 2);
        },
        None => panic!("Registry should exist"),
    }

    world.unregister_resource_from_trait::<dyn IncrementTrait, NumberValueResource2>();

    match world.get_resource::<TraitResourceRegistry<dyn IncrementTrait>>() {
        Some(registry) => {
            assert_eq!(registry.trait_data.len(), 1);
        },
        None => panic!("Registry should exist"),
    }

    world.unregister_resource_from_trait::<dyn IncrementTrait, NumberValueResource>();

    assert!(world.get_resource::<TraitResourceRegistry<dyn IncrementTrait>>().is_none());
    assert!(world.get_resource::<TraitResourceRegistry<dyn IncrementStringTrait>>().is_some());
}
