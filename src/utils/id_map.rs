//! An Identifier Map. An abstraction for structures who may have many names, but only
//! once instance.
//!
//! The Identifier Map, or `IdMap` for short, handles the case when a structure is stored
//! once, but may be retrieved using a variety of names or aliases. A naive approach would be
//! a simple `HashMap<String, Struct>`. However, this is inefficient. You would have to keep
//! copies of the structure for each of its name. To avoid this, the `IdMap` assigns a unique
//! *identifier* to the structure that is cheap to copy and small to store in memory. Consequently,
//! instead of keeping copies of the structure for each of its name, we do this for the identifier.
//! The structure can then be retrieved with the identifier. The `IdMap` employs two `HashMap`s to
//! accomplish its job. For small structures, `IdMap` might be inefficient memory-wise and
//! processor-wise. But it can pay off well for big/huge structures, which may be hundreds of bytes
//! long.
//!
//! The `IdMap` is generic. You can use it with any type for the name of the structure, the identifier
//! for the structure, and the structure itself.
//!
//! # Examples
//!
//! Using `IdMap` for your own purposes:
//!
//! ```rust
//! use serenity_standard_framework::utils::IdMap;
//!
//! #[derive(Debug, PartialEq)]
//! struct Foo {
//!     bar: i32,
//!     baz: String,
//! }
//!
//! #[derive(Clone, Copy, PartialEq, Eq, Hash)]
//! struct FooId(u64);
//!
//! let mut map: IdMap<String, FooId, Foo> = IdMap::new();
//!
//! let foo1 = Foo { bar: 1, baz: "2".to_string() };
//! let foo2 = Foo { bar: 3, baz: "4".to_string() };
//!
//! map.insert_name("fo".to_string(), FooId(1));
//! map.insert_name("foo".to_string(), FooId(1));
//! map.insert(FooId(1), foo1);
//!
//! map.insert_name("go".to_string(), FooId(2));
//! map.insert(FooId(2), foo2);
//!
//! assert_eq!(map.get(FooId(1)), Some(&Foo { bar: 1, baz: "2".to_string() }));
//! // This will panic if a structure under that identifier does not exist.
//! assert_eq!(&map[FooId(1)], &Foo { bar: 1, baz: "2".to_string() });
//! assert_eq!(map.get_by_name("fo"), Some(&Foo { bar: 1, baz: "2".to_string() }));
//! assert_eq!(map.get_by_name("foo"), Some(&Foo { bar: 1, baz: "2".to_string() }));
//!
//! assert_eq!(&map[FooId(2)], &Foo { bar: 3, baz: "4".to_string() });
//! assert_eq!(map.get_by_name("go"), Some(&Foo { bar: 3, baz: "4".to_string() }));
//! assert_eq!(map.get_by_name("goo"), None);
//! ```

use std::borrow::Borrow;
use std::collections::hash_map::{HashMap, IntoIter, Iter, IterMut, Keys, Values, ValuesMut};
use std::hash::Hash;
use std::ops::{Index, IndexMut};

/// An Identifier Map. An abstraction for structures who may have many names, but only
/// once instance.
///
/// Refer to the [module-level documentation][module]
///
/// [module]: index.html
#[derive(Debug, Clone)]
pub struct IdMap<Name, Id, Struct> {
    name_to_id: HashMap<Name, Id>,
    structures: HashMap<Id, Struct>,
}

impl<Name, Id, Struct> Default for IdMap<Name, Id, Struct> {
    fn default() -> Self {
        Self {
            name_to_id: HashMap::default(),
            structures: HashMap::default(),
        }
    }
}

impl<Name, Id, Struct> IdMap<Name, Id, Struct> {
    /// Creates a new `IdMap` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total number of names stored.
    pub fn len_names(&self) -> usize {
        self.name_to_id.len()
    }

    /// Returns the total number of structures stored.
    pub fn len(&self) -> usize {
        self.structures.len()
    }

    /// Returns the a boolean indicating that the map is empty.
    ///
    /// The map is regarded as empty when it contains no structures.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over all names stored in the map.
    pub fn iter_names(&self) -> Keys<'_, Name, Id> {
        self.name_to_id.keys()
    }

    /// Returns an iterator over all identifiers stored in the map.
    ///
    /// Duplicate identifiers may appear.
    pub fn iter_ids(&self) -> Values<'_, Name, Id> {
        self.name_to_id.values()
    }

    /// Returns an iterator over all structures and their assigned
    /// identifier.
    pub fn iter(&self) -> Iter<'_, Id, Struct> {
        self.structures.iter()
    }

    /// Returns a mutable iterator over all structures and their assigned
    /// identifier.
    ///
    /// Only the structures are mutable.
    pub fn iter_mut(&mut self) -> IterMut<'_, Id, Struct> {
        self.structures.iter_mut()
    }
}

impl<Name, Id, Struct> IdMap<Name, Id, Struct>
where
    Name: Hash + Eq,
    Id: Hash + Eq + Copy,
{
    /// Assigns a name to an identifier.
    ///
    /// Returns `None` if the name does not exist in the map.
    ///
    /// Returns `Some(old_id)` if the name exists in the map. The identifier
    /// is overwritten with the new identifier.
    pub fn insert_name(&mut self, name: Name, id: Id) -> Option<Id> {
        self.name_to_id.insert(name, id)
    }

    /// Retrieves an identifier based on a name.
    ///
    /// A copy of the identifier is returned.
    ///
    /// Returns `None` if an identifier does not belong to the name,
    /// otherwise `Some`.
    pub fn get_id<B: ?Sized>(&self, name: &B) -> Option<Id>
    where
        Name: Borrow<B>,
        B: Hash + Eq,
    {
        self.name_to_id.get(name).copied()
    }

    /// Retrieves a structure based on an identifier.
    ///
    /// An immutable reference to the structure is returned.
    ///
    /// Returns `None` if a structure does not belong to the name,
    /// otherwise `Some`.
    pub fn get_by_name<B: ?Sized>(&self, name: &B) -> Option<&Struct>
    where
        Name: Borrow<B>,
        B: Hash + Eq,
    {
        self.get_id(name).and_then(|id| self.structures.get(&id))
    }

    /// Retrieves a structure based on an identifier.
    ///
    /// A mutable reference to the structure is returned.
    ///
    /// Returns `None` if a structure does not belong to the name,
    /// otherwise `Some`.
    pub fn get_by_name_mut<B: ?Sized>(&mut self, name: &B) -> Option<&mut Struct>
    where
        Name: Borrow<B>,
        B: Hash + Eq,
    {
        self.get_id(name)
            .and_then(move |id| self.structures.get_mut(&id))
    }

    /// Retrieves both an identifier and its structure based on a name.
    ///
    /// An identifier and an immutable reference to the structure is returned.
    ///
    /// Returns `None` if a identifier/structure does not belong to the name,
    /// otherwise `Some`.
    pub fn get_pair<B: ?Sized>(&self, name: &B) -> Option<(Id, &Struct)>
    where
        Name: Borrow<B>,
        B: Hash + Eq,
    {
        let id = self.get_id(name)?;
        self.structures.get(&id).map(|aggr| (id, aggr))
    }

    /// Returns a boolean indicating that a structure exists under a name
    pub fn contains<B: ?Sized>(&self, name: &B) -> bool
    where
        Name: Borrow<B>,
        B: Hash + Eq,
    {
        match self.get_id(name) {
            Some(id) => self.structures.contains_key(name),
            None => false,
        }
    }
}

impl<Name, Id, Struct> IdMap<Name, Id, Struct>
where
    Id: Hash + Eq,
{
    /// Assigns a structure to an identifier.
    pub fn insert(&mut self, id: Id, aggr: Struct) -> Option<Struct> {
        self.structures.insert(id, aggr)
    }

    /// Retrieves a structure based on an identifier.
    ///
    /// An immutable reference is returned.
    ///
    /// Returns `None` if a structure does not belong to the identifier,
    /// otherwise `Some`.
    pub fn get(&self, id: Id) -> Option<&Struct> {
        self.structures.get(&id)
    }

    /// Retrieves a structure based on an identifier.
    ///
    /// An mutable reference is returned.
    ///
    /// Returns `None` if a structure does not belong to the identifier,
    /// otherwise `Some`.
    pub fn get_mut(&mut self, id: Id) -> Option<&mut Struct> {
        self.structures.get_mut(&id)
    }
}

impl<Name, Id, Struct> Index<Id> for IdMap<Name, Id, Struct>
where
    Id: Hash + Eq,
{
    type Output = Struct;

    fn index(&self, index: Id) -> &Self::Output {
        self.get(index).expect("ID with an associated structure")
    }
}

impl<Name, Id, Struct> IndexMut<Id> for IdMap<Name, Id, Struct>
where
    Id: Hash + Eq,
{
    fn index_mut(&mut self, index: Id) -> &mut Self::Output {
        self.get_mut(index)
            .expect("ID with an associated structure")
    }
}

impl<Name, Id, Struct> IntoIterator for IdMap<Name, Id, Struct> {
    type IntoIter = IntoIter<Id, Struct>;
    type Item = (Id, Struct);

    fn into_iter(self) -> Self::IntoIter {
        self.structures.into_iter()
    }
}

impl<'a, Name, Id, Struct> IntoIterator for &'a IdMap<Name, Id, Struct> {
    type IntoIter = Iter<'a, Id, Struct>;
    type Item = (&'a Id, &'a Struct);

    fn into_iter(self) -> Self::IntoIter {
        self.structures.iter()
    }
}

impl<'a, Name, Id, Struct> IntoIterator for &'a mut IdMap<Name, Id, Struct> {
    type IntoIter = IterMut<'a, Id, Struct>;
    type Item = (&'a Id, &'a mut Struct);

    fn into_iter(self) -> Self::IntoIter {
        self.structures.iter_mut()
    }
}
