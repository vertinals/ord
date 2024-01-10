use std::cell::Cell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use crate::Index;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum CacheTableIndex {
    TXID_TO_INSCRIPTION_RECEIPTS,

    BRC20_BALANCES,
    BRC20_TOKEN,
    BRC20_EVENTS,
    BRC20_TRANSFERABLELOG,
    BRC20_INSCRIBE_TRANSFER,
}

pub struct CacheStateWriteDB<'db, 'a> {
    ord: CacheWriter<'db, 'a>,
    btc: CacheWriter<'db, 'a>,
    brc20: CacheWriter<'db, 'a>,
    brc20s: CacheWriter<'db, 'a>,
    _phantom_a: PhantomData<&'a ()>,
    _phantom_db: PhantomData<&'db ()>,
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

#[derive(Clone)]
pub struct CacheWriter<'db, 'a> {
    cache: Rc<Cell<Option<MultiCache>>>,
    pub index: Arc<Index>,
    _phantom_a: PhantomData<&'a ()>,
    _phantom_db: PhantomData<&'db ()>,
}


pub fn string_to_bytes(s: &str) -> Vec<u8> {
    let byte_slice: &[u8] = s.as_bytes();
    let byte_vec: Vec<u8> = byte_slice.to_vec();
    byte_vec
}

impl<'db, 'a> CacheWriter<'db, 'a> {
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
        self.index.clone()
    }
}