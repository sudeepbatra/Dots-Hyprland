// Take a look at the license at the top of the repository in the LICENSE file.

use glib::prelude::*;
use glib::translate::{FromGlib, IntoGlib};
use glib::{gflags, GBoxed, GEnum, GErrorDomain, GSharedBoxed};

#[test]
fn derive_gerror_domain() {
    #[derive(Debug, Eq, PartialEq, Clone, Copy, GErrorDomain)]
    #[gerror_domain(name = "TestError")]
    enum TestError {
        Invalid,
        Bad,
        Wrong,
    }

    let err = glib::Error::new(TestError::Bad, "oh no!");
    assert!(err.is::<TestError>());
    assert!(matches!(err.kind::<TestError>(), Some(TestError::Bad)));
}

#[test]
fn derive_shared_arc() {
    #[derive(Debug, Eq, PartialEq, Clone)]
    struct MyInnerShared {
        foo: String,
    }
    #[derive(Debug, Eq, PartialEq, Clone, GSharedBoxed)]
    #[gshared_boxed(type_name = "MyShared")]
    struct MyShared(std::sync::Arc<MyInnerShared>);

    let t = MyShared::static_type();
    assert!(t.is_a(glib::Type::BOXED));
    assert_eq!(t.name(), "MyShared");

    let p = MyShared(std::sync::Arc::new(MyInnerShared {
        foo: String::from("bar"),
    }));

    assert_eq!(std::sync::Arc::strong_count(&p.0), 1);
    let v = p.to_value();
    assert_eq!(std::sync::Arc::strong_count(&p.0), 2);
    let p_clone = v.get::<MyShared>().unwrap();
    assert_eq!(std::sync::Arc::strong_count(&p.0), 3);
    drop(p_clone);
    assert_eq!(std::sync::Arc::strong_count(&p.0), 2);
    drop(v);
    assert_eq!(std::sync::Arc::strong_count(&p.0), 1);
}

#[test]
fn derive_shared_arc_nullable() {
    #[derive(Debug, Eq, PartialEq, Clone)]
    struct MyInnerNullableShared {
        foo: String,
    }
    #[derive(Clone, Debug, PartialEq, Eq, GSharedBoxed)]
    #[gshared_boxed(type_name = "MyNullableShared", nullable)]
    struct MyNullableShared(std::sync::Arc<MyInnerNullableShared>);

    let t = MyNullableShared::static_type();
    assert!(t.is_a(glib::Type::BOXED));
    assert_eq!(t.name(), "MyNullableShared");

    let p = MyNullableShared(std::sync::Arc::new(MyInnerNullableShared {
        foo: String::from("bar"),
    }));

    assert_eq!(std::sync::Arc::strong_count(&p.0), 1);
    let _v = p.to_value();
    assert_eq!(std::sync::Arc::strong_count(&p.0), 2);

    let p = Some(MyNullableShared(std::sync::Arc::new(
        MyInnerNullableShared {
            foo: String::from("foo"),
        },
    )));

    assert_eq!(std::sync::Arc::strong_count(&p.as_ref().unwrap().0), 1);
    let v = p.to_value();
    assert_eq!(std::sync::Arc::strong_count(&p.as_ref().unwrap().0), 2);
    assert_eq!(
        p.as_ref().unwrap().0.foo,
        v.get::<MyNullableShared>().unwrap().0.foo
    );

    let b: Option<&MyNullableShared> = None;
    let v = b.to_value();
    assert_eq!(None, v.get::<Option<MyNullableShared>>().unwrap());
}

#[test]
fn derive_genum() {
    #[derive(Debug, Eq, PartialEq, Clone, Copy, GEnum)]
    #[repr(u32)]
    #[genum(type_name = "TestAnimalType")]
    enum Animal {
        Goat,
        #[genum(name = "The Dog")]
        Dog,
        #[genum(name = "The Cat", nick = "chat")]
        Cat = 5,
        Badger,
    }

    assert_eq!(Animal::Goat.into_glib(), 0);
    assert_eq!(Animal::Dog.into_glib(), 1);
    assert_eq!(Animal::Cat.into_glib(), 5);

    assert_eq!(unsafe { Animal::from_glib(0) }, Animal::Goat);
    assert_eq!(unsafe { Animal::from_glib(1) }, Animal::Dog);
    assert_eq!(unsafe { Animal::from_glib(5) }, Animal::Cat);

    assert_eq!(Animal::Goat.to_value().get::<Animal>(), Ok(Animal::Goat));
    assert_eq!(Animal::Dog.to_value().get::<Animal>(), Ok(Animal::Dog));
    assert_eq!(Animal::Cat.to_value().get::<Animal>(), Ok(Animal::Cat));

    let t = Animal::static_type();
    assert!(t.is_a(glib::Type::ENUM));
    assert_eq!(t.name(), "TestAnimalType");

    let e = glib::EnumClass::new(t).expect("EnumClass::new failed");
    let v = e.value(0).expect("EnumClass::get_value(0) failed");
    assert_eq!(v.name(), "Goat");
    assert_eq!(v.nick(), "goat");
    let v = e.value(1).expect("EnumClass::get_value(1) failed");
    assert_eq!(v.name(), "The Dog");
    assert_eq!(v.nick(), "dog");
    let v = e.value(5).expect("EnumClass::get_value(5) failed");
    assert_eq!(v.name(), "The Cat");
    assert_eq!(v.nick(), "chat");
    assert_eq!(e.value(2), None);
}

#[test]
fn derive_gboxed() {
    #[derive(Clone, Debug, PartialEq, Eq, GBoxed)]
    #[gboxed(type_name = "MyBoxed")]
    struct MyBoxed(String);

    let t = MyBoxed::static_type();
    assert!(t.is_a(glib::Type::BOXED));
    assert_eq!(t.name(), "MyBoxed");

    let b = MyBoxed(String::from("abc"));
    let v = b.to_value();
    assert_eq!(&b, v.get::<&MyBoxed>().unwrap());
    assert_eq!(b, v.get::<MyBoxed>().unwrap());
}

#[test]
fn derive_gboxed_nullable() {
    #[derive(Clone, Debug, PartialEq, Eq, GBoxed)]
    #[gboxed(type_name = "MyNullableBoxed", nullable)]
    struct MyNullableBoxed(String);

    let t = MyNullableBoxed::static_type();
    assert!(t.is_a(glib::Type::BOXED));
    assert_eq!(t.name(), "MyNullableBoxed");

    let b = MyNullableBoxed(String::from("abc"));
    let v = b.to_value();
    assert_eq!(&b, v.get::<Option<&MyNullableBoxed>>().unwrap().unwrap());
    assert_eq!(b, v.get::<Option<MyNullableBoxed>>().unwrap().unwrap());

    let b = Some(MyNullableBoxed(String::from("def")));
    let v = b.to_value();
    let b = b.unwrap();
    assert_eq!(&b, v.get::<Option<&MyNullableBoxed>>().unwrap().unwrap());
    assert_eq!(b, v.get::<Option<MyNullableBoxed>>().unwrap().unwrap());

    let b = Some(MyNullableBoxed(String::from("def")));
    let v = (&b).to_value();
    let b = b.unwrap();
    assert_eq!(&b, v.get::<Option<&MyNullableBoxed>>().unwrap().unwrap());
    assert_eq!(b, v.get::<Option<MyNullableBoxed>>().unwrap().unwrap());

    let b: Option<MyNullableBoxed> = None;
    let v = b.to_value();
    assert_eq!(None, v.get::<Option<&MyNullableBoxed>>().unwrap());
    assert_eq!(None, v.get::<Option<MyNullableBoxed>>().unwrap());
}

#[test]
fn attr_gflags() {
    #[gflags("MyFlags")]
    enum MyFlags {
        #[gflags(name = "Flag A", nick = "nick-a")]
        A = 0b00000001,
        #[gflags(name = "Flag B")]
        B = 0b00000010,
        #[gflags(skip)]
        AB = Self::A.bits() | Self::B.bits(),
        C = 0b00000100,
    }

    assert_eq!(MyFlags::A.bits(), 1);
    assert_eq!(MyFlags::B.bits(), 2);
    assert_eq!(MyFlags::AB.bits(), 3);

    assert_eq!(MyFlags::empty().into_glib(), 0);
    assert_eq!(MyFlags::A.into_glib(), 1);
    assert_eq!(MyFlags::B.into_glib(), 2);
    assert_eq!(MyFlags::AB.into_glib(), 3);

    assert_eq!(unsafe { MyFlags::from_glib(0) }, MyFlags::empty());
    assert_eq!(unsafe { MyFlags::from_glib(1) }, MyFlags::A);
    assert_eq!(unsafe { MyFlags::from_glib(2) }, MyFlags::B);
    assert_eq!(unsafe { MyFlags::from_glib(3) }, MyFlags::AB);

    assert_eq!(
        MyFlags::empty().to_value().get::<MyFlags>(),
        Ok(MyFlags::empty())
    );
    assert_eq!(MyFlags::A.to_value().get::<MyFlags>(), Ok(MyFlags::A));
    assert_eq!(MyFlags::B.to_value().get::<MyFlags>(), Ok(MyFlags::B));
    assert_eq!(MyFlags::AB.to_value().get::<MyFlags>(), Ok(MyFlags::AB));

    let t = MyFlags::static_type();
    assert!(t.is_a(glib::Type::FLAGS));
    assert_eq!(t.name(), "MyFlags");

    let e = glib::FlagsClass::new(t).expect("FlagsClass::new failed");
    let v = e.value(1).expect("FlagsClass::get_value(1) failed");
    assert_eq!(v.name(), "Flag A");
    assert_eq!(v.nick(), "nick-a");
    let v = e.value(2).expect("FlagsClass::get_value(2) failed");
    assert_eq!(v.name(), "Flag B");
    assert_eq!(v.nick(), "b");
    let v = e.value(4).expect("FlagsClass::get_value(4) failed");
    assert_eq!(v.name(), "C");
    assert_eq!(v.nick(), "c");

    assert!(e.value_by_name("Flag A").is_some());
    assert!(e.value_by_name("Flag B").is_some());
    assert!(e.value_by_name("AB").is_none());
    assert!(e.value_by_name("C").is_some());

    assert!(e.value_by_nick("nick-a").is_some());
    assert!(e.value_by_nick("b").is_some());
    assert!(e.value_by_nick("ab").is_none());
    assert!(e.value_by_nick("c").is_some());
}

#[test]
fn subclassable() {
    mod foo {
        use super::*;
        use glib::subclass::prelude::*;

        mod imp {
            use super::*;

            #[derive(Default)]
            pub struct Foo {}

            #[glib::object_subclass]
            impl ObjectSubclass for Foo {
                const NAME: &'static str = "MyFoo";
                type Type = super::Foo;
                type ParentType = glib::Object;
            }

            impl ObjectImpl for Foo {}
        }

        pub trait FooExt: 'static {
            fn test(&self);
        }

        impl<O: IsA<Foo>> FooExt for O {
            fn test(&self) {
                let _self = imp::Foo::from_instance(self.as_ref().downcast_ref::<Foo>().unwrap());
                unimplemented!()
            }
        }

        glib::wrapper! {
            pub struct Foo(ObjectSubclass<imp::Foo>);
        }
    }
}
