use std::{
    borrow::Cow,
    collections::btree_map::{self, BTreeMap},
    fmt,
};

use crate::{
    ExtensionDescriptorRef, FieldDescriptorRef, KindRef, MessageDescriptorRef, OneofDescriptorRef,
    Value,
};

use super::unknown::UnknownField;

pub(super) trait FieldDescriptorLike<'a>: Copy + fmt::Debug {
    fn number(&self) -> u32;
    fn default_value(&self) -> Value;
    fn is_default_value(&self, value: &Value) -> bool;
    fn is_valid(&self, value: &Value) -> bool;
    fn containing_oneof(&self) -> Option<OneofDescriptorRef<'a>>;
    fn supports_presence(&self) -> bool;
    fn kind(&self) -> KindRef<'a>;
    fn is_group(&self) -> bool;
    fn is_list(&self) -> bool;
    fn is_map(&self) -> bool;
    fn is_packed(&self) -> bool;
    fn is_packable(&self) -> bool;
    fn has(&self, value: &Value) -> bool {
        self.supports_presence() || !self.is_default_value(value)
    }
}

/// A set of extension fields in a protobuf message.
#[derive(Default, Debug, Clone, PartialEq)]
pub(super) struct DynamicMessageFieldSet {
    fields: BTreeMap<u32, ValueOrUnknown>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum ValueOrUnknown {
    Value(Value),
    Unknown(Vec<UnknownField>),
}

pub(super) enum ValueAndDescriptor<'a> {
    Field(&'a Value, FieldDescriptorRef<'a>),
    Extension(&'a Value, ExtensionDescriptorRef<'a>),
    Unknown(u32, &'a [UnknownField]),
}

impl DynamicMessageFieldSet {
    fn get_value(&self, number: u32) -> Option<&Value> {
        match self.fields.get(&number) {
            Some(ValueOrUnknown::Value(value)) => Some(value),
            Some(ValueOrUnknown::Unknown(_)) | None => None,
        }
    }

    pub(super) fn has<'a>(&self, desc: impl FieldDescriptorLike<'a>) -> bool {
        self.get_value(desc.number())
            .map(|value| desc.has(value))
            .unwrap_or(false)
    }

    pub(super) fn get<'a>(&self, desc: impl FieldDescriptorLike<'a>) -> Cow<'_, Value> {
        match self.get_value(desc.number()) {
            Some(value) => Cow::Borrowed(value),
            None => Cow::Owned(desc.default_value()),
        }
    }

    pub(super) fn get_mut<'a>(&mut self, desc: impl FieldDescriptorLike<'a>) -> &mut Value {
        self.clear_oneof_fields(desc);
        match self.fields.entry(desc.number()) {
            btree_map::Entry::Occupied(entry) => match entry.into_mut() {
                ValueOrUnknown::Value(value) => value,
                value @ ValueOrUnknown::Unknown(_) => {
                    *value = ValueOrUnknown::Value(desc.default_value());
                    value.unwrap_value_mut()
                }
            },
            btree_map::Entry::Vacant(entry) => entry
                .insert(ValueOrUnknown::Value(desc.default_value()))
                .unwrap_value_mut(),
        }
    }

    pub(super) fn set<'a>(&mut self, desc: impl FieldDescriptorLike<'a>, value: Value) {
        debug_assert!(
            desc.is_valid(&value),
            "invalid value {:?} for field {:?}",
            value,
            desc,
        );

        self.clear_oneof_fields(desc);
        self.fields
            .insert(desc.number(), ValueOrUnknown::Value(value));
    }

    fn clear_oneof_fields<'a>(&mut self, desc: impl FieldDescriptorLike<'a>) {
        if let Some(oneof_desc) = desc.containing_oneof() {
            for oneof_field in oneof_desc.fields() {
                if oneof_field.number() != desc.number() {
                    self.clear(oneof_field);
                }
            }
        }
    }

    pub(crate) fn add_unknown(&mut self, number: u32, unknown: UnknownField) {
        match self.fields.entry(number) {
            btree_map::Entry::Occupied(mut entry) => match entry.get_mut() {
                ValueOrUnknown::Value(_) => {
                    panic!("expected no field to be found with number {}", number)
                }
                ValueOrUnknown::Unknown(unknowns) => unknowns.push(unknown),
            },
            btree_map::Entry::Vacant(entry) => {
                entry.insert(ValueOrUnknown::Unknown(vec![unknown]));
            }
        }
    }

    pub(super) fn clear<'a>(&mut self, desc: impl FieldDescriptorLike<'a>) {
        self.fields.remove(&desc.number());
    }

    pub(crate) fn iter<'a>(
        &'a self,
        message: MessageDescriptorRef<'a>,
    ) -> impl Iterator<Item = ValueAndDescriptor> + 'a {
        self.fields
            .iter()
            .filter_map(move |(&number, value)| match value {
                ValueOrUnknown::Value(value) => {
                    if let Some(field) = message.get_field(number) {
                        if field.has(value) {
                            Some(ValueAndDescriptor::Field(value, field))
                        } else {
                            None
                        }
                    } else if let Some(extension) = message.get_extension(number) {
                        if extension.has(value) {
                            Some(ValueAndDescriptor::Extension(value, extension))
                        } else {
                            None
                        }
                    } else {
                        panic!("no field found with number {}", number)
                    }
                }
                ValueOrUnknown::Unknown(unknown) => {
                    Some(ValueAndDescriptor::Unknown(number, unknown.as_slice()))
                }
            })
    }

    pub(super) fn clear_all(&mut self) {
        self.fields.clear();
    }
}

impl ValueOrUnknown {
    fn unwrap_value_mut(&mut self) -> &mut Value {
        match self {
            ValueOrUnknown::Value(value) => value,
            ValueOrUnknown::Unknown(_) => unreachable!(),
        }
    }
}

impl<'a> FieldDescriptorLike<'a> for FieldDescriptorRef<'a> {
    fn number(&self) -> u32 {
        self.number()
    }

    fn default_value(&self) -> Value {
        Value::default_value_for_field(*self)
    }

    fn is_default_value(&self, value: &Value) -> bool {
        value.is_default_for_field(*self)
    }

    fn is_valid(&self, value: &Value) -> bool {
        value.is_valid_for_field(*self)
    }

    fn containing_oneof(&self) -> Option<OneofDescriptorRef<'a>> {
        self.containing_oneof()
    }

    fn supports_presence(&self) -> bool {
        self.supports_presence()
    }

    fn kind(&self) -> KindRef<'a> {
        self.kind()
    }

    fn is_group(&self) -> bool {
        self.is_group()
    }

    fn is_list(&self) -> bool {
        self.is_list()
    }

    fn is_map(&self) -> bool {
        self.is_map()
    }

    fn is_packed(&self) -> bool {
        self.is_packed()
    }

    fn is_packable(&self) -> bool {
        self.is_packable()
    }
}

impl<'a> FieldDescriptorLike<'a> for ExtensionDescriptorRef<'a> {
    fn number(&self) -> u32 {
        self.number()
    }

    fn default_value(&self) -> Value {
        Value::default_value_for_extension(*self)
    }

    fn is_default_value(&self, value: &Value) -> bool {
        value.is_default_for_extension(*self)
    }

    fn is_valid(&self, value: &Value) -> bool {
        value.is_valid_for_extension(*self)
    }

    fn containing_oneof(&self) -> Option<OneofDescriptorRef<'a>> {
        None
    }

    fn supports_presence(&self) -> bool {
        self.supports_presence()
    }

    fn kind(&self) -> KindRef<'a> {
        self.kind()
    }

    fn is_group(&self) -> bool {
        self.is_group()
    }

    fn is_list(&self) -> bool {
        self.is_list()
    }

    fn is_map(&self) -> bool {
        self.is_map()
    }

    fn is_packed(&self) -> bool {
        self.is_packed()
    }

    fn is_packable(&self) -> bool {
        self.is_packable()
    }
}
