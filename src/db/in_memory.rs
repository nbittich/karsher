use crate::prelude::*;

pub const DEFAULT_TREE: &str = "__adana_default";

use super::{tree::Tree, DbOp, Key, Op, Value};

type InnerMap<K, V> = BTreeMap<String, Tree<K, V>>;

#[derive(Debug, Deserialize, Serialize)]
pub struct InMemoryDb<K: Key, V: Value> {
    trees: InnerMap<K, V>,
    default_tree: String,
    current_tree: Option<String>,
}

impl<K: Key + Clone, V: Value + Clone> InMemoryDb<K, V> {
    fn apply_to_current_tree<E, F: FnOnce(&mut Tree<K, V>) -> Option<E>>(
        &mut self,
        apply_fn: F,
    ) -> Option<E> {
        let current_tree = self.get_current_tree()?;

        let tree = self.trees.get_mut(&current_tree)?;

        apply_fn(tree)
    }
}

impl<K: Key + Clone, V: Value> Op<K, V> for InMemoryDb<K, V> {
    fn read(&self, k: impl Into<K>, r: impl Fn(&V) -> Option<V>) -> Option<V> {
        let current_tree = self.get_current_tree()?;

        let tree = self.trees.get(&current_tree)?;
        tree.read(k, r)
    }

    fn insert(&mut self, k: impl Into<K>, v: impl Into<V>) -> Option<V> {
        self.apply_to_current_tree(move |tree| tree.insert(k, v))
    }

    fn remove(&mut self, k: impl Into<K>) -> Option<V> {
        self.apply_to_current_tree(move |tree| tree.remove(k))
    }

    fn clear(&mut self) {
        self.apply_to_current_tree(move |tree| {
            tree.clear();
            Some(())
        });
    }

    fn contains(&self, k: &K) -> Option<bool> {
        let current_tree = self.get_current_tree()?;

        let tree = self.trees.get(&current_tree)?;
        tree.contains(k)
    }

    fn len(&self) -> Option<usize> {
        let current_tree = self.get_current_tree()?;

        let tree = self.trees.get(&current_tree)?;
        tree.len()
    }

    fn keys(&self) -> Vec<K> {
        self.get_current_tree()
            .and_then(|current_tree| self.trees.get(&current_tree))
            .iter()
            .flat_map(|tree| tree.keys())
            .collect()
    }

    fn list_all(&self) -> BTreeMap<K, V> {
        self.get_current_tree()
            .and_then(|current_tree| self.trees.get(&current_tree))
            .iter()
            .flat_map(|tree| tree.list_all())
            .collect()
    }
}

impl<K: Key + Clone, V: Value + Clone> Default for InMemoryDb<K, V> {
    fn default() -> InMemoryDb<K, V> {
        let default_tree = DEFAULT_TREE.to_string();

        let mut db = InMemoryDb {
            trees: BTreeMap::new(),
            default_tree,
            current_tree: None,
        };
        db.open_tree(DEFAULT_TREE);

        db
    }
}

impl<K: Key + Clone, V: Value + Clone> DbOp<K, V> for InMemoryDb<K, V> {
    fn get_current_tree(&self) -> Option<String> {
        self.current_tree
            .as_ref()
            .cloned()
            .or_else(|| Some(self.default_tree.to_string()))
    }

    /// return true if the three has to be opened
    fn open_tree(&mut self, tree_name: &str) -> Option<bool> {
        if let Some(current_tree) = &self.current_tree {
            if current_tree == tree_name {
                return Some(false);
            }
        }
        if self.trees.get(tree_name).is_none() {
            self.trees.insert(tree_name.to_string(), Tree::default());
        }

        let _ = self.current_tree.insert(tree_name.to_string());
        Some(true)
    }

    fn tree_names(&self) -> Vec<String> {
        self.trees.keys().map(|s| s.to_string()).collect()
    }

    fn drop_tree(&mut self, tree_name: &str) -> bool {
        if tree_name == self.default_tree {
            return self.clear_tree(tree_name);
        }
        let _ = self.current_tree.take();

        self.trees.remove(tree_name).is_some()
    }

    fn clear_tree(&mut self, tree_name: &str) -> bool {
        if let Some(tree) = self.trees.get_mut(tree_name) {
            tree.clear();
            true
        } else {
            false
        }
    }

    fn merge_trees(
        &mut self,
        tree_name_source: &str,
        tree_name_dest: &str,
    ) -> Option<()> {
        let source = self.trees.remove(tree_name_source)?;
        let dest = self.trees.get_mut(tree_name_dest)?;
        dest.extend(source.iter().map(|(k, v)| (k.clone(), v.clone())));
        let _ = self.trees.insert(tree_name_source.to_string(), source);

        Some(())
    }

    fn merge_current_tree_with(
        &mut self,
        tree_name_source: &str,
    ) -> Option<()> {
        let current_tree =
            self.get_current_tree().filter(|t| t != tree_name_source)?;
        self.merge_trees(tree_name_source, &current_tree)
    }

    fn apply_batch(&mut self, batch: super::Batch<K, V>) -> Option<()> {
        for op in batch.into_iter() {
            match op {
                super::OpType::Insert((k, v)) => {
                    self.insert(k, v);
                }
            }
        }
        Some(())
    }

    fn apply_tree(
        &mut self,
        tree_name: &str,
        consumer: &mut impl FnMut(&mut Tree<K, V>) -> Option<V>,
    ) -> Option<V> {
        let tree = self.trees.get_mut(tree_name)?;
        consumer(tree)
    }
}

#[cfg(test)]
mod test {

    use crate::db::in_memory::DEFAULT_TREE;
    use crate::db::DbOp;
    use crate::prelude::*;

    use super::{InMemoryDb, Op};

    #[derive(Serialize, Debug, PartialEq, Clone)]
    struct MyString(String);

    #[test]
    fn test_inner_db() {
        let mut db: InMemoryDb<String, Box<MyString>> = InMemoryDb::default();
        db.insert("babsa", Box::new(MyString("babuch".into())));
        assert_eq!(Some(1), db.len());

        db.read("babsa", |v| {
            assert_eq!(v, &Box::new(MyString("babuch".into())));
            None
        });

        db.open_tree("test");
        assert_eq!(Some(0), db.len());

        assert_eq!(db.get_current_tree(), Some("test".to_string()));

        db.insert("baba", Box::new(MyString("babuch".into())));
        assert_eq!(Some(1), db.len()); // len of the current tree todo maybe it should be better to provide the len of the actual db

        let value = db.read("baba", |v| Some(v.clone()));

        assert_eq!(value, Some(Box::new(MyString("babuch".to_string()))));

        db.drop_tree("test");

        assert_eq!(db.get_current_tree(), Some(DEFAULT_TREE.to_string()));

        assert_eq!(Some(1), db.len());

        db.clear();

        assert_eq!(Some(0), db.len()); // now default is cleared

        assert_eq!(false, db.clear_tree("bababa")); // bababa doesn't exist

        let mut db: InMemoryDb<String, String> = InMemoryDb::default();

        db.insert("baba", "babuch");
        db.insert("bibi", "bobo");
        assert_eq!(Some(2), db.len());

        db.remove("baba");
        assert_eq!(Some(1), db.len());

        db.drop_tree(DEFAULT_TREE);
        assert_eq!(Some(0), db.len());

        db.open_tree("rust");
        db.insert("fmt", "cargo fmt");

        let value = db.read("fmt", |v| Some(v.to_string()));

        assert_eq!(Some("cargo fmt".into()), value);

        let mut db: InMemoryDb<i32, String> = InMemoryDb::default();

        db.insert(22, "babuch");
        db.insert(44, "bobo");

        db.open_tree("rust");
        assert_eq!(db.get_current_tree(), Some("rust".to_string()));

        db.insert(443, "cargo fmt");

        db.read_no_op(66, |v| {
            println!("{v:?}");
            Some(v.to_string())
        });
        let value = db.read(443, |v| Some(v.to_string()));

        assert_eq!(Some("cargo fmt".into()), value);

        let mut db: InMemoryDb<i32, i32> = InMemoryDb::default();
        assert_eq!(Some(0), db.len());

        db.insert(22, 3333);
        db.insert(44, 4745);
        db.insert(22, 5555);
        db.insert(44, 9999);
        assert_eq!(Some(2), db.len());

        db.insert(99, 3333);
        db.insert(55, 4745);
        assert_eq!(Some(4), db.len());

        db.open_tree("rust");
        assert_eq!(Some(0), db.len());

        db.insert(443, 333);
        assert_eq!(Some(1), db.len());
        let value = db.read(999555, |v| Some(*v));

        assert_eq!(None, value);

        let value = db.read(443, |v| Some(*v));
        assert_eq!(Some(333), value);
    }
}
