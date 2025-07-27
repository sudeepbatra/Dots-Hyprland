// Take a look at the license at the top of the repository in the LICENSE file.

use glib::Cast;

use glib::translate::*;

use glib::subclass::prelude::*;

use crate::Application;
use crate::Window;

pub trait GtkApplicationImpl:
    GtkApplicationImplExt + gio::subclass::prelude::ApplicationImpl
{
    fn window_added(&self, application: &Self::Type, window: &Window) {
        self.parent_window_added(application, window)
    }

    fn window_removed(&self, application: &Self::Type, window: &Window) {
        self.parent_window_removed(application, window)
    }
}

pub trait GtkApplicationImplExt: ObjectSubclass {
    fn parent_window_added(&self, application: &Self::Type, window: &Window);
    fn parent_window_removed(&self, application: &Self::Type, window: &Window);
}

impl<T: GtkApplicationImpl> GtkApplicationImplExt for T {
    fn parent_window_added(&self, application: &Self::Type, window: &Window) {
        unsafe {
            let data = T::type_data();
            let parent_class = data.as_ref().parent_class() as *mut ffi::GtkApplicationClass;
            if let Some(f) = (*parent_class).window_added {
                f(
                    application
                        .unsafe_cast_ref::<Application>()
                        .to_glib_none()
                        .0,
                    window.to_glib_none().0,
                )
            }
        }
    }

    fn parent_window_removed(&self, application: &Self::Type, window: &Window) {
        unsafe {
            let data = T::type_data();
            let parent_class = data.as_ref().parent_class() as *mut ffi::GtkApplicationClass;
            if let Some(f) = (*parent_class).window_removed {
                f(
                    application
                        .unsafe_cast_ref::<Application>()
                        .to_glib_none()
                        .0,
                    window.to_glib_none().0,
                )
            }
        }
    }
}

unsafe impl<T: GtkApplicationImpl> IsSubclassable<T> for Application {
    fn class_init(class: &mut ::glib::Class<Self>) {
        unsafe extern "C" fn application_window_added<T: GtkApplicationImpl>(
            ptr: *mut ffi::GtkApplication,
            wptr: *mut ffi::GtkWindow,
        ) {
            let instance = &*(ptr as *mut T::Instance);
            let imp = instance.impl_();
            let wrap: Borrowed<Application> = from_glib_borrow(ptr);

            imp.window_added(wrap.unsafe_cast_ref(), &from_glib_borrow(wptr))
        }
        unsafe extern "C" fn application_window_removed<T: GtkApplicationImpl>(
            ptr: *mut ffi::GtkApplication,
            wptr: *mut ffi::GtkWindow,
        ) {
            let instance = &*(ptr as *mut T::Instance);
            let imp = instance.impl_();
            let wrap: Borrowed<Application> = from_glib_borrow(ptr);

            imp.window_removed(wrap.unsafe_cast_ref(), &from_glib_borrow(wptr))
        }

        unsafe extern "C" fn application_startup<T: GtkApplicationImpl>(
            ptr: *mut gio::ffi::GApplication,
        ) {
            let instance = &*(ptr as *mut T::Instance);
            let imp = instance.impl_();
            let wrap: Borrowed<gio::Application> = from_glib_borrow(ptr);
            crate::rt::set_initialized();
            imp.startup(wrap.unsafe_cast_ref())
        }

        <gio::Application as IsSubclassable<T>>::class_init(class);

        let klass = class.as_mut();
        klass.window_added = Some(application_window_added::<T>);
        klass.window_removed = Some(application_window_removed::<T>);
        // Chain our startup handler in here
        let klass = &mut class.as_mut().parent_class;
        klass.startup = Some(application_startup::<T>);
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <gio::Application as IsSubclassable<T>>::instance_init(instance);
    }
}
