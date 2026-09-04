use std::{fmt::Debug, hash::Hash, marker::PhantomData};

pub struct Handle<Marker> {
    raw: usize,
    _marker: PhantomData<Marker>,
}
pub trait HandleMarker {
    const NAME: &'static str;
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Handle<T> {}
impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}
impl<T> Eq for Handle<T> {}

impl<T> Handle<T> {
    pub fn null() -> Self {
        Self {
            raw: usize::MAX,
            _marker: Default::default(),
        }
    }

    pub fn is_null(&self) -> bool {
        self.eq(&Self::null())
    }

    pub fn from_raw(raw: usize) -> Self {
        Self {
            raw,
            _marker: Default::default(),
        }
    }
    pub fn as_raw(&self) -> usize {
        self.raw
    }
}

impl<T> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}
impl<T> Debug for Handle<T>
where
    T: HandleMarker,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle<{}>({:#x})", T::NAME, self.raw,)
    }
}

enum Slot<Data> {
    Occupied(Data),
    Free { next_free_idx: Option<usize> },
}
#[derive(Default)]
pub struct HandleMap<Handle, Data> {
    slots: Vec<Slot<Data>>,
    first_free_idx: Option<usize>,
    _marker: PhantomData<Handle>,
}
impl<Marker, Data> Default for HandleMap<Handle<Marker>, Data> {
    fn default() -> Self {
        Self {
            slots: Default::default(),
            first_free_idx: Default::default(),
            _marker: Default::default(),
        }
    }
}
impl<Marker, Data> HandleMap<Handle<Marker>, Data> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, data: Data) -> Handle<Marker> {
        if let Some(idx) = self.first_free_idx {
            if let Slot::Free { next_free_idx } = self.slots[idx] {
                self.first_free_idx = next_free_idx;
                self.slots[idx] = Slot::Occupied(data);
                Handle::from_raw(idx)
            } else {
                unreachable!("Data corruption: Expected Slot::Free");
            }
        } else {
            let idx = self.slots.len();
            self.slots.push(Slot::Occupied(data));
            Handle::from_raw(idx)
        }
    }

    pub fn remove(&mut self, handle: Handle<Marker>) -> Option<Data> {
        let idx = handle.as_raw();
        if idx >= self.slots.len() {
            return None;
        }

        if let Slot::Occupied(_) = self.slots[idx] {
            if let Slot::Occupied(data) = std::mem::replace(
                &mut self.slots[idx],
                Slot::Free {
                    next_free_idx: self.first_free_idx,
                },
            ) {
                self.first_free_idx = Some(idx);
                Some(data)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.slots.clear();
        self.first_free_idx = None;
    }

    pub fn get(&self, handle: Handle<Marker>) -> Option<&Data> {
        match self.slots.get(handle.as_raw()) {
            Some(Slot::Occupied(data)) => Some(data),
            _ => None,
        }
    }
    pub fn get_mut(&mut self, handle: Handle<Marker>) -> Option<&mut Data> {
        match self.slots.get_mut(handle.as_raw()) {
            Some(Slot::Occupied(data)) => Some(data),
            _ => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Handle<Marker>, &Data)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(idx, slot)| match slot {
                Slot::Occupied(data) => Some((Handle::<Marker>::from_raw(idx), data)),
                Slot::Free { .. } => None,
            })
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Handle<Marker>, &mut Data)> {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(idx, slot)| match slot {
                Slot::Occupied(data) => Some((Handle::<Marker>::from_raw(idx), data)),
                Slot::Free { .. } => None,
            })
    }
}

pub mod h_pastey {
    pub use pastey::paste;
}

#[macro_export]
macro_rules! define_handle {
    {$name:ident} => {
        // 1. Paste and concatenate the identifier to create a clean, lowercase module name
        utils131::h_pastey::paste! {
            mod [< handle_ $name:lower >] {
                use utils131::{ HandleMarker, Handle };

                pub struct [< $name HandleMarker >];

                impl HandleMarker for [< $name HandleMarker >] {
                    const NAME: &'static str = stringify!($name);
                }
            }

            // 2. Export the public type alias using the generated marker
            pub type [< $name Handle >] = utils131::Handle<[< handle_ $name:lower >]::[< $name HandleMarker >]>;
        }
    };
}
