// Take a look at the license at the top of the repository in the LICENSE file.

use glib::subclass::prelude::*;

use super::container::ContainerImpl;
use crate::Container;
use crate::Stack;

pub trait StackImpl: ContainerImpl {}

unsafe impl<T: ContainerImpl> IsSubclassable<T> for Stack {
    fn class_init(class: &mut ::glib::Class<Self>) {
        <Container as IsSubclassable<T>>::class_init(class);
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <Container as IsSubclassable<T>>::instance_init(instance);
    }
}
