use std::fmt::{Debug, Display};

/// A location within a [`Value`].
///
/// Conceptually, it is a list of choices that one has to make to go to a certain place within
/// the value. In practice, it is used to locate the origin of a deserialization error.
///
/// ## Example
/// ```
/// use deserr::ValuePointerRef;
///
/// let pointer = ValuePointerRef::Origin;
/// let pointer = pointer.push_key("a");
/// let pointer = pointer.push_index(2);
/// // now `pointer` points to "a".2
/// ```
///
/// A `ValuePointerRef` is an immutable data structure, so it is cheap to extend and to copy.
/// However, if you want to store it inside an owned type, you may want to convert it to a
/// [`ValuePointer`] instead using [`self.to_owned()`](ValuePointerRef::to_owned).
#[derive(Clone, Copy)]
pub enum ValuePointerRef<'a> {
    Origin,
    Key {
        key: &'a str,
        prev: &'a ValuePointerRef<'a>,
    },
    Index {
        index: usize,
        prev: &'a ValuePointerRef<'a>,
    },
}

impl<'a> Default for ValuePointerRef<'a> {
    fn default() -> Self {
        Self::Origin
    }
}

impl<'a> ValuePointerRef<'a> {
    /// Extend `self` such that it points to the next subvalue at the given `key`.
    #[must_use]
    pub fn push_key(&'a self, key: &'a str) -> Self {
        Self::Key { key, prev: self }
    }

    #[must_use]
    /// Extend `self` such that it points to the next subvalue at the given index.
    pub fn push_index(&'a self, index: usize) -> Self {
        Self::Index { index, prev: self }
    }

    /// Return true if the pointer is at the origin.
    pub fn is_origin(&self) -> bool {
        match self {
            ValuePointerRef::Origin => true,
            _ => false,
        }
    }

    /// Return the last field encountered if there is one.
    pub fn last_field(&self) -> Option<&str> {
        match self {
            ValuePointerRef::Origin => None,
            ValuePointerRef::Key { key, .. } => Some(key),
            ValuePointerRef::Index { prev, .. } => prev.last_field(),
        }
    }

    /// Convert `self` to its owned version
    pub fn to_owned(&self) -> ValuePointer {
        let mut cur = self;
        let mut components = vec![];
        loop {
            match cur {
                ValuePointerRef::Origin => break,
                ValuePointerRef::Key { key, prev } => {
                    components.push(ValuePointerComponent::Key(key.to_string()));
                    cur = prev;
                }
                ValuePointerRef::Index { index, prev } => {
                    components.push(ValuePointerComponent::Index(*index));
                    cur = prev;
                }
            }
        }
        let components = components.into_iter().rev().collect();
        ValuePointer { path: components }
    }
}

/// Part of a [`ValuePointer`]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValuePointerComponent {
    Key(String),
    Index(usize),
}

/// The owned version of a [`ValuePointerRef`].
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValuePointer {
    pub path: Vec<ValuePointerComponent>,
}

/// Equivalent to [`Value`] but without the associated data.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Null,
    Boolean,
    Integer,
    NegativeInteger,
    Float,
    String,
    Sequence,
    Map,
}

impl Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueKind::Null => write!(f, "Null"),
            ValueKind::Boolean => write!(f, "Boolean"),
            ValueKind::Integer => write!(f, "Integer"),
            ValueKind::NegativeInteger => write!(f, "NegativeInteger"),
            ValueKind::Float => write!(f, "Float"),
            ValueKind::String => write!(f, "String"),
            ValueKind::Sequence => write!(f, "Sequence"),
            ValueKind::Map => write!(f, "Map"),
        }
    }
}

impl Debug for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

/// `Value<V>` is a view into the parsed serialization data (of type `V`) that
/// is readable by Deserr.
///
/// It is an enum with a variant for each possible value kind. The content of the variants
/// is either a simple value, such as `bool` or `String`, or an abstract [`Sequence`] or
/// [`Map`], which are views into the rest of the serialized data.
#[derive(Debug)]
pub enum Value<V: IntoValue> {
    Null,
    Boolean(bool),
    Integer(u64),
    NegativeInteger(i64),
    Float(f64),
    String(String),
    Sequence(V::Sequence),
    Map(V::Map),
}

impl<V: IntoValue> Value<V> {
    pub fn kind(&self) -> ValueKind {
        match self {
            Value::Null => ValueKind::Null,
            Value::Boolean(_) => ValueKind::Boolean,
            Value::Integer(_) => ValueKind::Integer,
            Value::NegativeInteger(_) => ValueKind::NegativeInteger,
            Value::Float(_) => ValueKind::Float,
            Value::String(_) => ValueKind::String,
            Value::Sequence(_) => ValueKind::Sequence,
            Value::Map(_) => ValueKind::Map,
        }
    }
}

/// A trait for a value that can be deserialized via [`DeserializeFromValue`].
pub trait IntoValue: Sized {
    type Sequence: Sequence<Value = Self>;
    type Map: Map<Value = Self>;

    fn kind(&self) -> ValueKind;
    fn into_value(self) -> Value<Self>;
}

/// A sequence of values conforming to [`IntoValue`].
pub trait Sequence {
    type Value: IntoValue;
    type Iter: Iterator<Item = Self::Value>;

    fn len(&self) -> usize;
    fn into_iter(self) -> Self::Iter;
}

/// A keyed map of values conforming to [`IntoValue`].
pub trait Map {
    type Value: IntoValue;
    type Iter: Iterator<Item = (String, Self::Value)>;

    fn len(&self) -> usize;
    fn remove(&mut self, key: &str) -> Option<Self::Value>;
    fn into_iter(self) -> Self::Iter;
}
