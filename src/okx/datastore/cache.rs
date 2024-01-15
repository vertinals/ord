use std::cell::Cell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use crate::Index;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum CacheTableIndex {
    TXID_TO_INSCRIPTION_RECEIPTS,

    SEQUENCE_NUMBER_TO_SATPOINT,
    SAT_TO_SEQUENCE_NUMBER,
    HOME_INSCRIPTIONS,
    INSCRIPTION_ID_TO_SEQUENCE_NUMBER,
    SEQUENCE_NUMBER_TO_CHILDREN,
    SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY,
    INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER,
    OUTPOINT_TO_ENTRY,

    BRC20_BALANCES,
    BRC20_TOKEN,
    BRC20_EVENTS,
    BRC20_TRANSFERABLELOG,
    BRC20_INSCRIBE_TRANSFER,
    ORD_TX_TO_OPERATIONS,
    COLLECTIONS_KEY_TO_INSCRIPTION_ID,
    COLLECTIONS_INSCRIPTION_ID_TO_KINDS,
}

#[derive(Clone)]
pub struct MultiCache {
    caches: HashMap<CacheTableIndex, CacheTable>,
}

#[derive(Clone)]
pub enum KeyPrefix {}

#[derive(Clone, Default)]
pub struct CacheTable {
    pub data: HashMap<Vec<u8>, Vec<u8>>,
}

impl CacheTable {
    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.data.insert(key, value);
    }
    pub fn pop_first(&mut self) {}
    pub fn get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        self.data.get(&key).map_or(None, |v| {
            Some(v.clone())
        })
    }
    pub fn remove(&mut self, key: Vec<u8>) {
        self.data.remove(&key);
    }
}

#[derive(Clone)]
pub struct CacheWriter {
    cache: Rc<Cell<Option<MultiCache>>>,
    pub internal_index: Arc<Index>,
}


pub fn string_to_bytes(s: &str) -> Vec<u8> {
    let byte_slice: &[u8] = s.as_bytes();
    let byte_vec: Vec<u8> = byte_slice.to_vec();
    byte_vec
}

impl CacheWriter {
    pub fn use_cache<T>(&self, table: CacheTableIndex, f: impl FnOnce(Option<&CacheTable>) -> T) -> T {
        let cache = self.cache.take();
        let m_cache = cache.as_ref().unwrap();
        let table_cache = m_cache.caches.get(&table);
        let ret = f(table_cache);
        self.cache.set(cache);
        ret
    }
    pub fn use_cache_mut<T>(&self, table: CacheTableIndex, f: impl FnOnce(&mut CacheTable) -> T) -> T {
        let mut cache = self.cache.take();
        let mut m_cache = cache.as_mut().unwrap();
        let mut table_cache = m_cache.caches.get_mut(&table);
        if table_cache.is_none() {
            let internal_table = CacheTable::default();
            m_cache.caches.insert(table.clone(), internal_table);
            table_cache = m_cache.caches.get_mut(&table);
        }

        let ret = f(table_cache.as_mut().unwrap());
        self.cache.set(cache);
        ret
    }
    pub fn get_index(&self) -> Arc<Index> {
        self.internal_index.clone()
    }
    pub fn new(internal_index: Arc<Index>) -> Self {
        Self { cache: Default::default(), internal_index }
    }
}