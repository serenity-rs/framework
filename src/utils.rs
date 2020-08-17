use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct IdMap<Name, Id, Aggr> {
    name_to_id: HashMap<Name, Id>,
    aggregrates: HashMap<Id, Aggr>,
}

impl<Name, Id, Aggr> Default for IdMap<Name, Id, Aggr> {
    fn default() -> Self {
        Self {
            name_to_id: HashMap::default(),
            aggregrates: HashMap::default(),
        }
    }
}

impl<Name, Id, Aggr> IdMap<Name, Id, Aggr> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len_names(&self) -> usize {
        self.name_to_id.len()
    }

    pub fn len(&self) -> usize {
        self.aggregrates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<Name, Id, Aggr> IdMap<Name, Id, Aggr>
where
    Name: Hash + Eq,
    Id: Hash + Eq + Copy,
{
    pub fn insert_name<I>(&mut self, name: I, id: Id) -> Option<Id>
    where
        I: Into<Name>,
    {
        self.name_to_id.insert(name.into(), id)
    }

    pub fn get_id<B: ?Sized>(&self, name: &B) -> Option<Id>
    where
        Name: Borrow<B>,
        B: Hash + Eq,
    {
        self.name_to_id.get(name).copied()
    }

    pub fn get_by_name<B: ?Sized>(&self, name: &B) -> Option<&Aggr>
    where
        Name: Borrow<B>,
        B: Hash + Eq,
    {
        self.get_id(name).and_then(|id| self.aggregrates.get(&id))
    }

    pub fn get_by_name_mut<B: ?Sized>(&mut self, name: &B) -> Option<&mut Aggr>
    where
        Name: Borrow<B>,
        B: Hash + Eq,
    {
        self.get_id(name)
            .and_then(move |id| self.aggregrates.get_mut(&id))
    }
}

impl<Name, Id, Aggr> IdMap<Name, Id, Aggr>
where
    Id: Hash + Eq,
{
    pub fn insert(&mut self, id: Id, aggr: Aggr) -> Option<Aggr> {
        self.aggregrates.insert(id, aggr)
    }

    pub fn get(&self, id: Id) -> Option<&Aggr> {
        self.aggregrates.get(&id)
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Aggr> {
        self.aggregrates.get_mut(&id)
    }
}

impl<Name, Id, Aggr> Index<Id> for IdMap<Name, Id, Aggr>
where
    Id: Hash + Eq,
{
    type Output = Aggr;

    fn index(&self, index: Id) -> &Self::Output {
        self.get(index).expect("ID with an associated aggregate")
    }
}

impl<Name, Id, Aggr> IndexMut<Id> for IdMap<Name, Id, Aggr>
where
    Id: Hash + Eq,
{
    fn index_mut(&mut self, index: Id) -> &mut Self::Output {
        self.get_mut(index)
            .expect("ID with an associated aggregate")
    }
}
