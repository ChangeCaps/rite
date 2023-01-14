use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Index, IndexMut},
};

const ID_REPLACEMENTS: &[(&str, &str)] = &[
    ("ritec_hir::module::Module", "hir::Module"),
    ("ritec_hir::function::Function", "hir::Function"),
    ("ritec_hir::local::Local", "hir::Local"),
    ("ritec_hir::expr::Expr", "hir::Expr"),
    ("ritec_hir::stmt::Stmt", "hir::Stmt"),
    ("ritec_mir::local::Local", "mir::Local"),
    ("ritec_mir_build::thir::expr::Expr", "thir::Expr"),
    ("ritec_mir_build::thir::stmt::Stmt", "thir::Stmt"),
];

pub struct Id<T> {
    index: usize,
    marker: PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    pub const fn from_raw_index(index: usize) -> Self {
        Self {
            index,
            marker: PhantomData,
        }
    }

    pub const fn as_raw_index(self) -> usize {
        self.index
    }

    pub const fn cast<U>(self) -> Id<U> {
        Id {
            index: self.index,
            marker: PhantomData,
        }
    }

    pub fn name(&self) -> String {
        let mut name = String::from(std::any::type_name::<T>());

        for (from, to) in ID_REPLACEMENTS {
            name = name.replace(from, to);
        }

        format!("{}[{}]", name, self.index)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Id {
            index: self.index,
            marker: PhantomData,
        }
    }
}

impl<T> Copy for Id<T> {}

impl<T> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Arena<T> {
    arena: Vec<Option<T>>,
    free: Vec<usize>,
}

impl<T> Arena<T> {
    pub const fn new() -> Self {
        Arena {
            arena: Vec::new(),
            free: Vec::new(),
        }
    }

    /// Inserts an item into the arena, returning the id of the item.
    #[inline]
    pub fn reserve(&mut self) -> Id<T> {
        if let Some(index) = self.free.pop() {
            Id::from_raw_index(index)
        } else {
            let index = self.arena.len();
            self.arena.push(None);
            Id::from_raw_index(index)
        }
    }

    /// Inserts an `item` into the arena at the given `id`.
    pub fn insert(&mut self, id: Id<T>, item: T) -> Option<T> {
        if id.index >= self.arena.len() {
            self.arena.resize_with(id.index + 1, || None);
        }

        let slot = self.arena.get_mut(id.index)?;
        slot.replace(item)
    }

    /// Inserts an `item` into the arena, returning the [`Id`] of the item.
    #[inline]
    pub fn push(&mut self, item: T) -> Id<T> {
        if let Some(index) = self.free.pop() {
            self.arena[index] = Some(item);

            Id::from_raw_index(index)
        } else {
            let index = self.arena.len();
            self.arena.push(Some(item));

            Id::from_raw_index(index)
        }
    }

    /// Removes the item at the arena without freeing the id.
    #[inline]
    pub fn take(&mut self, id: Id<T>) -> Option<T> {
        self.arena.get_mut(id.as_raw_index())?.take()
    }

    /// Remove an item from the arena, this will free the id for reuse.
    #[inline]
    pub fn remove(&mut self, id: Id<T>) -> Option<T> {
        let item = self.take(id);

        if item.is_some() {
            self.free.push(id.as_raw_index());
        }

        item
    }

    #[inline]
    pub fn get(&self, id: Id<T>) -> Option<&T> {
        self.arena.get(id.as_raw_index())?.as_ref()
    }

    #[inline]
    pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
        self.arena.get_mut(id.as_raw_index())?.as_mut()
    }

    /// Gets the id of an item in the arena.
    ///
    /// *Note* this is an O(n) operation and should be used with care.
    #[inline]
    pub fn get_id(&self, item: &T) -> Option<Id<T>>
    where
        T: PartialEq,
    {
        for (i, v) in self.arena.iter().enumerate() {
            if let Some(v) = v {
                if v == item {
                    return Some(Id::from_raw_index(i));
                }
            }
        }

        None
    }

    #[inline]
    pub fn keys(&self) -> impl DoubleEndedIterator<Item = Id<T>> + '_ {
        self.arena.iter().enumerate().filter_map(|(i, v)| {
            if v.is_some() {
                Some(Id::from_raw_index(i))
            } else {
                None
            }
        })
    }

    #[inline]
    pub fn values(&self) -> impl DoubleEndedIterator<Item = &T> {
        self.arena.iter().filter_map(|v| v.as_ref())
    }

    #[inline]
    pub fn values_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> {
        self.arena.iter_mut().filter_map(|v| v.as_mut())
    }

    #[inline]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (Id<T>, &T)> {
        self.arena
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.as_ref().map(|v| (Id::from_raw_index(i), v)))
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = (Id<T>, &mut T)> {
        self.arena
            .iter_mut()
            .enumerate()
            .filter_map(|(i, v)| v.as_mut().map(|v| (Id::from_raw_index(i), v)))
    }

    #[inline]
    pub fn clear(&mut self) {
        self.arena.clear();
        self.free.clear();
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            arena: Default::default(),
            free: Default::default(),
        }
    }
}

impl<T> Index<Id<T>> for Arena<T> {
    type Output = T;

    #[track_caller]
    fn index(&self, index: Id<T>) -> &Self::Output {
        self.arena[index.index].as_ref().expect("invalid id")
    }
}

impl<T> IndexMut<Id<T>> for Arena<T> {
    fn index_mut(&mut self, index: Id<T>) -> &mut Self::Output {
        self.arena[index.index].as_mut().expect("invalid id")
    }
}

impl<T: Debug> Debug for Arena<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.iter().map(|(id, v)| (id, v)))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::Arena;

    #[test]
    fn insert() {
        let mut arena = Arena::new();

        let id = arena.reserve();
        arena.insert(id, 1);

        assert_eq!(arena[id], 1);
    }

    #[test]
    fn push() {
        let mut arena = Arena::new();

        let id = arena.push(1);

        assert_eq!(arena[id], 1);
    }

    #[test]
    fn reserve() {
        let mut arena = Arena::new();

        let id = arena.reserve();
        assert!(arena.get(id).is_none());

        arena.insert(id, 1);
        assert_eq!(arena[id], 1);

        assert_eq!(arena.remove(id), Some(1));
        assert!(arena.get(id).is_none());
    }
}
