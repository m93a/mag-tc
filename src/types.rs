use std::{any::Any, ops::Shl, ptr::eq, sync::Arc};

pub trait IntoAny {
    fn into_any(&self) -> Box<&dyn Any>;
}
impl<T: Sized + 'static> IntoAny for T {
    fn into_any(&self) -> Box<&dyn Any> {
        Box::new(self)
    }
}

pub trait Type: IntoAny {
    fn is_assignable_to(&self, other: &dyn Type) -> bool;

    fn is_supertype_of(&self, _: Box<&dyn Type>) -> Option<bool> {
        None
    }
}

pub trait BoxableType {
    fn boxed(&self) -> Box<&dyn Type>;
    fn arc(self) -> Arc<dyn Type>;
}
impl<T: Type + Sized + 'static> BoxableType for T {
    fn boxed(&self) -> Box<&dyn Type> {
        Box::new(self)
    }
    fn arc(self) -> Arc<dyn Type> {
        Arc::new(self)
    }
}
impl BoxableType for Arc<dyn Type> {
    fn boxed(&self) -> Box<&dyn Type> {
        Box::new(self.as_ref())
    }
    fn arc(self) -> Arc<dyn Type> {
        self
    }
}
pub trait BoxedTypeUtils {
    fn cast<T: 'static>(&self) -> Option<&T>;
}
impl BoxedTypeUtils for Box<&dyn Type> {
    fn cast<T: 'static>(&self) -> Option<&T> {
        (*self.as_ref()).into_any().downcast_ref()
    }
}
impl BoxedTypeUtils for &dyn Type {
    fn cast<T: 'static>(&self) -> Option<&T> {
        (*self).into_any().downcast_ref()
    }
}

impl Shl for &dyn Type {
    type Output = bool;
    fn shl(self, rhs: Self) -> bool {
        rhs.is_assignable_to(self)
    }
}
impl Shl for Box<&dyn Type> {
    type Output = bool;
    fn shl(self, rhs: Self) -> bool {
        rhs.is_assignable_to(*self.as_ref())
    }
}
impl Shl for &Box<&dyn Type> {
    type Output = bool;
    fn shl(self, rhs: Self) -> bool {
        rhs.is_assignable_to(*self.as_ref())
    }
}

#[derive(PartialEq)]
pub enum Primitive {
    Void,
    Boolean,
    Int(usize),
    UInt(usize),
}
impl Type for Primitive {
    fn is_assignable_to(&self, other: &dyn Type) -> bool {
        if let Some(x) = other.is_supertype_of(self.boxed()) {
            return x;
        }
        if let Some(x) = other.cast::<Primitive>() {
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
    pub fn new(sup: Vec<&Arc<Trait>>) -> Trait {
        Trait {
            supertraits: sup.into_iter().cloned().collect(),
        }
    }
    pub fn new_arc(sup: Vec<&Arc<Trait>>) -> Arc<Trait> {
        Arc::new(Trait::new(sup))
    }
}
impl Type for Trait {
    fn is_assignable_to(&self, other: &dyn Type) -> bool {
        if let Some(x) = other.is_supertype_of(self.boxed()) {
            return x;
        }
        if let Some(x) = other.cast::<Trait>() {
            eq(self, x) || self.supertraits.iter().any(|s| s.is_assignable_to(x))
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

impl Function {
    fn new(args: Vec<(&str, Arc<dyn Type>)>, ret: Arc<dyn Type>) -> Function {
        Function {
            args: args
                .into_iter()
                .map(|(name, arg_type)| FunctionArgument {
                    name: name.into(),
                    arg_type,
                })
                .collect(),
            return_type: ret,
        }
    }
}

impl Type for Function {
    fn is_assignable_to(&self, other: &dyn Type) -> bool {
        if let Some(x) = other.is_supertype_of(self.boxed()) {
            println!("supertype");
            return x;
        }
        if let Some(x) = other.cast::<Function>() {
            if self.args.len() != x.args.len()
                || !self.return_type.is_assignable_to(x.return_type.as_ref())
            {
                return false;
            }

            for i in 0..self.args.len() {
                // args are contravariant
                if !x.args[i]
                    .arg_type
                    .is_assignable_to(self.args[i].arg_type.as_ref())
                {
                    return false;
                }
            }

            return true;
        }
        false
    }
}

#[test]
fn primitive_type() {
    let bool_a = Primitive::Boolean.boxed();
    let bool_b = Primitive::Boolean.boxed();
    let int32a = Primitive::Int(32).boxed();
    let int32b = Primitive::Int(32).boxed();
    let int64 = Primitive::Int(64).boxed();

    assert!(&bool_a << &bool_a);
    assert!(&bool_a << &bool_b);
    assert!(&bool_b << &bool_a);

    assert!(&int32a << &int32a);
    assert!(&int32a << &int32b);
    assert!(&int32b << &int32a);

    assert!(!(&bool_a << &int32a));
    assert!(!(&int32a << &bool_a));
    assert!(!(&int32a << &int64));
}

#[test]
fn trait_type() {
    let life = Trait::new_arc(vec![]);
    let plant = Trait::new_arc(vec![&life]);
    let animal = Trait::new_arc(vec![&life]);
    let dog = Trait::new_arc(vec![&animal]);

    let meower = Trait::new_arc(vec![]);
    let cat = Trait::new_arc(vec![&animal, &meower]);

    for t in [&life, &plant, &animal, &dog, &meower, &cat] {
        assert!(t.is_assignable_to(t.as_ref()));
    }

    assert!(plant.is_assignable_to(life.as_ref()));
    assert!(!life.is_assignable_to(plant.as_ref()));

    assert!(animal.is_assignable_to(life.as_ref()));
    assert!(dog.is_assignable_to(life.as_ref()));
    assert!(dog.is_assignable_to(animal.as_ref()));
    assert!(!dog.is_assignable_to(plant.as_ref()));

    assert!(cat.is_assignable_to(life.as_ref()));
    assert!(cat.is_assignable_to(animal.as_ref()));
    assert!(cat.is_assignable_to(meower.as_ref()));
    assert!(!cat.is_assignable_to(dog.as_ref()));
}

#[test]
fn function_type() {
    let animal = Trait::new_arc(vec![]);
    let cat = Trait::new_arc(vec![&animal]);

    let greet_animal = Function::new(vec![("a", animal.clone())], Primitive::Void.arc());
    let greet_cat = Function::new(vec![("c", cat.clone())], Primitive::Void.arc());
    assert!(greet_animal.is_assignable_to(&greet_animal));
    assert!(greet_animal.is_assignable_to(&greet_cat));
    assert!(greet_cat.is_assignable_to(&greet_cat));
    assert!(!greet_cat.is_assignable_to(&greet_animal));

    let get_animal = Function::new(vec![], animal.clone());
    let get_cat = Function::new(vec![], cat.clone());
    assert!(get_cat.is_assignable_to(&get_cat));
    assert!(get_cat.is_assignable_to(&get_animal));
    assert!(get_animal.is_assignable_to(&get_animal));
    assert!(!get_animal.is_assignable_to(&get_cat));

    let trade_animal = Function::new(vec![("a", animal.clone())], animal.clone());
    let trade_cat = Function::new(vec![("c", cat.clone())], cat.clone());
    assert!(trade_animal.is_assignable_to(&trade_animal));
    assert!(trade_cat.is_assignable_to(&trade_cat));
    assert!(!trade_animal.is_assignable_to(&trade_cat));
    assert!(!trade_cat.is_assignable_to(&trade_animal));
}
