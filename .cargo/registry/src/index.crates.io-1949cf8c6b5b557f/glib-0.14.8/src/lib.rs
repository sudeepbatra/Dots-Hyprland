// Take a look at the license at the top of the repository in the LICENSE file.

//! # Rust GLib and GObject bindings
//!
//! This library contains bindings to GLib and GObject types and APIs as well as
//! common building blocks used in both handmade and machine generated
//! bindings to GTK and other GLib-based libraries.
//!
//! It is the foundation for higher level libraries with uniform Rusty (safe and
//! strongly typed) APIs. It avoids exposing GLib-specific data types where
//! possible and is not meant to provide comprehensive GLib bindings, which
//! would often amount to duplicating the Rust Standard Library or other utility
//! crates.
//!
//! GLib 2.48 is the lowest supported version for the underlying library.
//!
//! # Dynamic typing
//!
//! Most types in the GLib family have [`Type`] identifiers.
//! Their corresponding Rust types implement the [`StaticType`] trait.
//!
//! A dynamically typed [`Value`] can carry values of any [`StaticType`].
//! [`Variant`]s can carry values of [`StaticVariantType`].
//!
//! # Errors
//!
//! Errors are represented by [`Error`], which can
//! carry values from various [error domains](error::ErrorDomain) such as
//! [`FileError`].
//!
//! # Objects
//!
//! Each class and interface has a corresponding smart pointer struct
//! representing an instance of that type (e.g. [`Object`] for `GObject` or
//! `gtk::Widget` for `GtkWidget`). They are reference counted and feature
//! interior mutability similarly to Rust's `Rc<RefCell<T>>` idiom.
//! Consequently, cloning objects is cheap and their methods never require
//! mutable borrows. Two smart pointers are equal if they point to the same
//! object.
//!
//! The root of the object hierarchy is [`Object`].
//! Inheritance and subtyping is denoted with the [`IsA`]
//! marker trait. The [`Cast`] trait enables upcasting
//! and downcasting.
//!
//! Interfaces and non-leaf classes also have corresponding traits (e.g.
//! [`ObjectExt`] or `gtk::WidgetExt`), which are blanketly implemented for all
//! their subtypes.
//!
//! You can create new subclasses of [`Object`] or other object types. Look at
//! the module's documentation for further details and a code example.
//!
//! # Under the hood
//!
//! GLib-based libraries largely operate on pointers to various boxed or
//! reference counted structures so the bindings have to implement corresponding
//! smart pointers (wrappers), which encapsulate resource management and safety
//! checks. Such wrappers are defined via the
//! [`macro@wrapper`] macro, which uses abstractions
//! defined in the [`mod@wrapper`], [`mod@boxed`],
//! [`mod@shared`] and [`mod@object`] modules.
//!
//! The [`mod@translate`] module defines and partly implements
//! conversions between high level Rust types (including the aforementioned
//! wrappers) and their FFI counterparts.

#![cfg_attr(feature = "dox", feature(doc_cfg))]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::wrong_self_convention)]

pub use ffi;
pub use gobject_ffi;

#[doc(hidden)]
pub use bitflags;

#[doc(hidden)]
pub use once_cell;

pub use glib_macros::{
    clone, gflags, object_interface, object_subclass, Downgrade, GBoxed, GEnum, GErrorDomain,
    GSharedBoxed,
};

pub use self::array::Array;
pub use self::byte_array::ByteArray;
pub use self::bytes::Bytes;
pub use self::closure::Closure;
pub use self::error::{BoolError, Error};
pub use self::file_error::FileError;
pub use self::object::{
    Cast, Class, InitiallyUnowned, Interface, IsA, Object, ObjectExt, ObjectType, SendWeakRef,
    WeakRef,
};
pub use self::signal::{
    signal_handler_block, signal_handler_disconnect, signal_handler_unblock,
    signal_stop_emission_by_name, SignalHandlerId,
};
pub use self::string::String;
use std::ffi::CStr;

pub use self::enums::{EnumClass, EnumValue, FlagsBuilder, FlagsClass, FlagsValue, UserDirectory};
pub use self::types::{ILong, StaticType, Type, ULong};
pub use self::value::{BoxedValue, SendValue, ToSendValue, ToValue, Value};
pub use self::variant::{FromVariant, StaticVariantType, ToVariant, Variant};
pub use self::variant_dict::VariantDict;
pub use self::variant_iter::{VariantIter, VariantStrIter};
pub use self::variant_type::{VariantTy, VariantType};

pub mod clone;
#[macro_use]
pub mod wrapper;
#[macro_use]
pub mod boxed;
#[macro_use]
pub mod shared;
#[macro_use]
pub mod error;
#[macro_use]
pub mod object;

pub use self::auto::functions::*;
pub use self::auto::*;
#[allow(non_upper_case_globals)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
#[allow(unused_imports)]
mod auto;

pub use self::gobject::*;
mod gobject;

mod array;
mod byte_array;
mod bytes;
pub mod char;
mod string;
pub use self::char::*;
mod checksum;
pub mod closure;
mod enums;
mod file_error;
mod functions;
pub use self::functions::*;
mod key_file;
pub mod prelude;
pub mod signal;
pub mod source;
pub use self::source::*;
#[macro_use]
pub mod translate;
mod gstring;
pub use self::gstring::GString;
pub mod types;
mod unicollate;
pub use self::unicollate::{CollationKey, FilenameCollationKey};
mod utils;
pub use self::utils::*;
mod main_context;
mod main_context_channel;
pub use self::main_context::MainContextAcquireGuard;
pub use self::main_context_channel::{Receiver, Sender, SyncSender};
mod date;
pub mod value;
pub mod variant;
mod variant_dict;
mod variant_iter;
mod variant_type;
pub use self::date::Date;
mod value_array;
pub use self::value_array::ValueArray;
mod param_spec;
pub use self::param_spec::*;
mod quark;
pub use self::quark::Quark;
#[macro_use]
mod log;
pub use self::log::log_set_handler;

// #[cfg(any(feature = "v2_50", feature = "dox"))]
// pub use log::log_variant;
pub use self::log::{
    log_default_handler, log_remove_handler, log_set_always_fatal, log_set_default_handler,
    log_set_fatal_mask, log_unset_default_handler, set_print_handler, set_printerr_handler,
    unset_print_handler, unset_printerr_handler, LogHandlerId, LogLevel, LogLevels,
};

#[doc(hidden)]
#[cfg(any(feature = "dox", feature = "log_macros"))]
#[cfg_attr(feature = "dox", doc(cfg(feature = "log_macros")))]
pub use rs_log;

#[cfg(any(feature = "log", feature = "dox"))]
#[macro_use]
mod bridged_logging;
#[cfg(any(feature = "log", feature = "dox"))]
pub use self::bridged_logging::{rust_log_handler, GlibLogger, GlibLoggerDomain, GlibLoggerFormat};

pub mod send_unique;
pub use self::send_unique::{SendUnique, SendUniqueCell};

#[macro_use]
pub mod subclass;

mod main_context_futures;
mod source_futures;
pub use self::source_futures::*;

mod thread_pool;
pub use self::thread_pool::ThreadPool;

/// This is the log domain used by the [`clone!`][crate::clone!] macro. If you want to use a custom
/// logger (it prints to stdout by default), you can set your own logger using the corresponding
/// `log` functions.
pub const CLONE_MACRO_LOG_DOMAIN: &str = "glib-rs-clone";

// Actual thread IDs can be reused by the OS once the old thread finished.
// This works around it by using our own counter for threads.
//
// Taken from the fragile crate
use std::sync::atomic::{AtomicUsize, Ordering};
fn next_thread_id() -> usize {
    static mut COUNTER: AtomicUsize = AtomicUsize::new(0);
    unsafe { COUNTER.fetch_add(1, Ordering::SeqCst) }
}

#[doc(alias = "get_thread_id")]
pub(crate) fn thread_id() -> usize {
    thread_local!(static THREAD_ID: usize = next_thread_id());
    THREAD_ID.with(|&x| x)
}

pub(crate) struct ThreadGuard<T> {
    thread_id: usize,
    value: T,
}

impl<T> ThreadGuard<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            thread_id: thread_id(),
            value,
        }
    }

    pub(crate) fn get_ref(&self) -> &T {
        if self.thread_id != thread_id() {
            panic!("Value accessed from different thread than where it was created");
        }

        &self.value
    }

    pub(crate) fn get_mut(&mut self) -> &mut T {
        if self.thread_id != thread_id() {
            panic!("Value accessed from different thread than where it was created");
        }

        &mut self.value
    }
}

impl<T> Drop for ThreadGuard<T> {
    fn drop(&mut self) {
        if self.thread_id != thread_id() {
            panic!("Value dropped on a different thread than where it was created");
        }
    }
}

unsafe impl<T> Send for ThreadGuard<T> {}

#[cfg(target_family = "windows")]
mod win32;

#[cfg(target_family = "windows")]
pub use self::win32::*;
