// Take a look at the license at the top of the repository in the LICENSE file.

use crate::ColorButton;
use crate::Widget;
use glib::object::Cast;
use glib::object::IsA;
use glib::translate::*;
use std::mem;

pub trait ColorButtonExtManual: 'static {
    #[doc(alias = "gtk_color_button_new_with_color")]
    fn with_color(color: &gdk::Color) -> ColorButton;

    #[doc(alias = "gtk_color_button_get_color")]
    #[doc(alias = "get_color")]
    fn color(&self) -> gdk::Color;

    #[doc(alias = "gtk_color_button_set_color")]
    fn set_color(&self, color: &gdk::Color);
}

impl<O: IsA<ColorButton>> ColorButtonExtManual for O {
    fn with_color(color: &gdk::Color) -> ColorButton {
        assert_initialized_main_thread!();
        unsafe { Widget::from_glib_none(ffi::gtk_color_button_new_with_color(color)).unsafe_cast() }
    }

    fn color(&self) -> gdk::Color {
        unsafe {
            let mut color = mem::MaybeUninit::uninit();
            ffi::gtk_color_button_get_color(self.as_ref().to_glib_none().0, color.as_mut_ptr());
            color.assume_init()
        }
    }

    fn set_color(&self, color: &gdk::Color) {
        unsafe { ffi::gtk_color_button_set_color(self.as_ref().to_glib_none().0, color) }
    }
}
