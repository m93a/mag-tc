use std::{any::Any, ptr::eq, sync::Arc};

pub trait Type {
    fn is_subtype_of(&self, other: Box<&dyn Type>) -> bool;
    fn box_(&self) -> Box<&dyn Type>;
    fn into_any(&self) -> Box<&dyn Any>;

    fn is_supertype_of(&self, other: Box<&dyn Type>) -> bool {
        other.is_subtype_of(self.box_())
    }
}

#[derive(PartialEq)]
pub enum Primitive {
    Boolean,
    Int(usize),
    UInt(usize),
}
impl Type for Primitive {
    fn box_(&self) -> Box<&dyn Type> {
        Box::new(self)
    }
    fn into_any(&self) -> Box<&dyn Any> {
        Box::new(self)
    }
    fn is_subtype_of(&self, other: Box<&dyn Type>) -> bool {
        if let Some(x) = other.into_any().downcast_ref::<Primitive>() {
            x == self
        } else {
            false
        }
    }
}

pub struct Trait {
    pub supertraits: Vec<Arc<Trait>>,
}
impl Trait {
    pub fn new(sup: Vec<Arc<Trait>>) -> Trait {
        Trait { supertraits: sup }
    }
}
impl Type for Trait {
    fn box_(&self) -> Box<&dyn Type> {
        Box::new(self)
    }
    fn into_any(&self) -> Box<&dyn Any> {
        Box::new(self)
    }
    fn is_subtype_of(&self, other: Box<&dyn Type>) -> bool {
        if let Some(x) = other.into_any().downcast_ref::<Trait>() {
            eq(self, x) || self.supertraits.iter().any(|s| s.is_subtype_of(x.box_()))
        } else {
            false
        }
    }
}

pub struct FunctionArgument {
    pub name: Box<str>,
    pub arg_type: Arc<dyn Type>,
}

pub struct Function {
    pub args: Vec<FunctionArgument>,
    pub return_type: Arc<dyn Type>,
}

#[test]
fn primitive_type() {
    let bool_a = Primitive::Boolean;
    let bool_b = Primitive::Boolean;
    let int32a = Primitive::Int(32);
    let int32b = Primitive::Int(32);
    let int64 = Primitive::Int(64);

    assert!(bool_a.is_subtype_of(bool_a.box_()));
    assert!(bool_a.is_subtype_of(bool_b.box_()));
    assert!(bool_b.is_subtype_of(bool_a.box_()));

    assert!(int32a.is_subtype_of(int32a.box_()));
    assert!(int32a.is_subtype_of(int32b.box_()));
    assert!(int32b.is_subtype_of(int32a.box_()));

    assert!(!bool_a.is_subtype_of(int32a.box_()));
    assert!(!int32a.is_subtype_of(bool_a.box_()));
    assert!(!int32a.is_subtype_of(int64.box_()));
}

#[test]
fn trait_type() {
    let life: Arc<Trait> = Trait::new(vec![]).into();
    let plant: Arc<Trait> = Trait::new(vec![life.clone()]).into();
    let animal: Arc<Trait> = Trait::new(vec![life.clone()]).into();
    let dog: Arc<Trait> = Trait::new(vec![animal.clone()]).into();

    let meower: Arc<Trait> = Trait::new(vec![]).into();
    let cat: Arc<Trait> = Trait::new(vec![animal.clone(), meower.clone()]).into();

    for t in [&life, &plant, &animal, &dog, &meower, &cat] {
        assert!(t.is_subtype_of(t.box_()));
    }

    assert!(plant.is_subtype_of(life.box_()));
    assert!(!life.is_subtype_of(plant.box_()));

    assert!(animal.is_subtype_of(life.box_()));
    assert!(dog.is_subtype_of(life.box_()));
    assert!(dog.is_subtype_of(animal.box_()));
    assert!(!dog.is_subtype_of(plant.box_()));

    assert!(cat.is_subtype_of(life.box_()));
    assert!(cat.is_subtype_of(animal.box_()));
    assert!(cat.is_subtype_of(meower.box_()));
    assert!(!cat.is_subtype_of(dog.box_()));
}
