#![doc = include_str!("../doc/intro.md")]
#![doc = "# Example - decoding"]
#![doc = include_str!("../doc/decoding.md")]
#![cfg_attr(feature = "serde", doc = "# Example - JSON mapping")]
#![cfg_attr(feature = "serde", doc = include_str!("../doc/json.md"))]
#![cfg_attr(feature = "derive", doc = "# Implementing [`ReflectMessage`]")]
#![cfg_attr(feature = "derive", doc = include_str!("../doc/reflect.md"))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations)]
// #![warn(missing_docs)]
#![deny(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/prost-reflect/0.8.1/")]

#[cfg(feature = "serde1")]
extern crate serde1 as serde;

mod descriptor;
mod dynamic;
mod reflect;

pub use {prost, prost::bytes, prost_types};

pub use self::descriptor::{
    Cardinality, DescriptorError, DescriptorPool, DescriptorPoolRef, EnumDescriptor,
    EnumDescriptorRef, EnumValueDescriptor, EnumValueDescriptorRef, ExtensionDescriptor,
    ExtensionDescriptorRef, FieldDescriptor, FieldDescriptorRef, FileDescriptor, FileDescriptorRef,
    Kind, KindRef, MessageDescriptor, MessageDescriptorRef, MethodDescriptor, MethodDescriptorRef,
    OneofDescriptor, OneofDescriptorRef, ServiceDescriptor, ServiceDescriptorRef, Syntax,
};
pub use self::dynamic::{DynamicMessage, MapKey, Value};
pub use self::reflect::ReflectMessage;

#[cfg(feature = "serde")]
pub use self::dynamic::{DeserializeOptions, SerializeOptions};

#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use prost_reflect_derive::ReflectMessage;
