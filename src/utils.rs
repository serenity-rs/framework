use std::borrow::Borrow;
use std::collections::hash_map::{HashMap, IntoIter, Iter, IterMut, Keys, Values, ValuesMut};
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

    pub fn iter_names(&self) -> Keys<'_, Name, Id> {
        self.name_to_id.keys()
    }

    pub fn iter_ids(&self) -> Values<'_, Name, Id> {
        self.name_to_id.values()
    }

    pub fn iter_ids_mut(&mut self) -> ValuesMut<'_, Name, Id> {
        self.name_to_id.values_mut()
    }

    pub fn iter(&self) -> Iter<'_, Id, Aggr> {
        self.aggregrates.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Id, Aggr> {
        self.aggregrates.iter_mut()
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

    pub fn get_pair<B: ?Sized>(&self, name: &B) -> Option<(Id, &Aggr)>
    where
        Name: Borrow<B>,
        B: Hash + Eq,
    {
        let id = self.get_id(name)?;
        self.aggregrates.get(&id).map(|aggr| (id, aggr))
    }

    pub fn get_by_name_mut<B: ?Sized>(&mut self, name: &B) -> Option<&mut Aggr>
    where
        Name: Borrow<B>,
        B: Hash + Eq,
    {
        self.get_id(name)
            .and_then(move |id| self.aggregrates.get_mut(&id))
    }

    pub fn contains<B: ?Sized>(&self, name: &B) -> bool
    where
        Name: Borrow<B>,
        B: Hash + Eq,
    {
        self.name_to_id.contains_key(name)
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

impl<Name, Id, Aggr> IntoIterator for IdMap<Name, Id, Aggr> {
    type IntoIter = IntoIter<Id, Aggr>;
    type Item = (Id, Aggr);

    fn into_iter(self) -> Self::IntoIter {
        self.aggregrates.into_iter()
    }
}

impl<'a, Name, Id, Aggr> IntoIterator for &'a IdMap<Name, Id, Aggr> {
    type IntoIter = Iter<'a, Id, Aggr>;
    type Item = (&'a Id, &'a Aggr);

    fn into_iter(self) -> Self::IntoIter {
        self.aggregrates.iter()
    }
}

impl<'a, Name, Id, Aggr> IntoIterator for &'a mut IdMap<Name, Id, Aggr> {
    type IntoIter = IterMut<'a, Id, Aggr>;
    type Item = (&'a Id, &'a mut Aggr);

    fn into_iter(self) -> Self::IntoIter {
        self.aggregrates.iter_mut()
    }
}
