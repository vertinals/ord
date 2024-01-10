use bitcoin::Txid;
use crate::index::{BRC20_BALANCES, BRC20_EVENTS, BRC20_INSCRIBE_TRANSFER, BRC20_TOKEN, BRC20_TRANSFERABLELOG};
use crate::InscriptionId;
use crate::okx::datastore::brc20::{Balance, Brc20Reader, Brc20ReaderWriter, Receipt, Tick, TokenInfo, TransferableLog, TransferInfo};
use crate::okx::datastore::brc20::redb::script_tick_key;
use crate::okx::datastore::brc20::redb::table::{get_balance, get_balances, get_inscribe_transfer_inscription, get_token_info, get_tokens_info, get_transaction_receipts, get_transferable, get_transferable_by_id, get_transferable_by_tick};
use crate::okx::datastore::cache::{CacheTableIndex, CacheWriter, string_to_bytes};
use crate::okx::datastore::ScriptKey;

pub struct BRC20CacheWrapper(CacheWriter);


impl BRC20CacheWrapper {}

impl Brc20Reader for BRC20CacheWrapper {
    type Error = anyhow::Error;

    fn get_balances(&self, script_key: &ScriptKey) -> crate::Result<Vec<Balance>, Self::Error> {
        let value: Option<Vec<Balance>> = self.0.use_cache(CacheTableIndex::BRC20_BALANCES, |maybe_cache| {
            if maybe_cache.is_none() {
                return None;
            }
            todo!()
        });
        if value.is_none() {
            let index = self.0.get_index();
            let rtx = index.begin_read()?;
            let table = rtx.0.open_table(BRC20_BALANCES)?;
            return get_balances(&table, script_key);
        }

        todo!()
    }

    fn get_balance(&self, script_key: &ScriptKey, tick: &Tick) -> crate::Result<Option<Balance>, Self::Error> {
        let value: Option<Balance> = self.0.use_cache(CacheTableIndex::BRC20_BALANCES, |maybe_cache| {
            if maybe_cache.is_none() {
                return None;
            }
            todo!()
        });
        if value.is_none() {
            let index = self.0.get_index();
            let rtx = index.begin_read()?;
            let table = rtx.0.open_table(BRC20_BALANCES)?;
            return get_balance(&table, script_key, tick);
        }
        todo!()
    }

    fn get_token_info(&self, tick: &Tick) -> crate::Result<Option<TokenInfo>, Self::Error> {
        let value: Option<TokenInfo> = self.0.use_cache(CacheTableIndex::BRC20_TOKEN, |maybe_cache| {
            if maybe_cache.is_none() {
                return None;
            }
            todo!()
        });
        if value.is_none() {
            let index = self.0.get_index();
            let rtx = index.begin_read()?;
            let table = rtx.0.open_table(BRC20_TOKEN)?;
            return get_token_info(&table, tick);
        }
        todo!()
    }

    fn get_tokens_info(&self) -> crate::Result<Vec<TokenInfo>, Self::Error> {
        let value: Option<Vec<TokenInfo>> = self.0.use_cache(CacheTableIndex::BRC20_TOKEN, |maybe_cache| {
            if maybe_cache.is_none() {
                return None;
            }
            todo!()
        });
        if value.is_none() {
            let index = self.0.get_index();
            let rtx = index.begin_read()?;
            let table = rtx.0.open_table(BRC20_TOKEN)?;
            return get_tokens_info(&table);
        }
        todo!()
    }

    fn get_transaction_receipts(&self, txid: &Txid) -> crate::Result<Vec<Receipt>, Self::Error> {
        let value: Option<Vec<Receipt>> = self.0.use_cache(CacheTableIndex::BRC20_EVENTS, |maybe_cache| {
            if maybe_cache.is_none() {
                return None;
            }
            todo!()
        });
        if value.is_none() {
            let index = self.0.get_index();
            let rtx = index.begin_read()?;
            let table = rtx.0.open_table(BRC20_EVENTS)?;
            return get_transaction_receipts(&table, txid);
        }

        todo!()
    }

    fn get_transferable(&self, script: &ScriptKey) -> crate::Result<Vec<TransferableLog>, Self::Error> {
        let value: Option<Vec<TransferableLog>> = self.0.use_cache(CacheTableIndex::BRC20_TRANSFERABLELOG, |maybe_cache| {
            if maybe_cache.is_none() {
                return None;
            }
            todo!()
        });
        if value.is_none() {
            let index = self.0.get_index();
            let rtx = index.begin_read()?;
            let table = rtx.0.open_table(BRC20_TRANSFERABLELOG)?;
            return get_transferable(&table, script);
        }
        todo!()
    }

    fn get_transferable_by_tick(&self, script: &ScriptKey, tick: &Tick) -> crate::Result<Vec<TransferableLog>, Self::Error> {
        let value: Option<Vec<TransferableLog>> = self.0.use_cache(CacheTableIndex::BRC20_TRANSFERABLELOG, |maybe_cache| {
            if maybe_cache.is_none() {
                return None;
            }
            todo!()
        });
        if value.is_none() {
            let index = self.0.get_index();
            let rtx = index.begin_read()?;
            let table = rtx.0.open_table(BRC20_TRANSFERABLELOG)?;
            return get_transferable_by_tick(&table, script, tick);
        }
        todo!()
    }

    fn get_transferable_by_id(&self, script: &ScriptKey, inscription_id: &InscriptionId) -> crate::Result<Option<TransferableLog>, Self::Error> {
        let value: Option<TransferableLog> = self.0.use_cache(CacheTableIndex::BRC20_TRANSFERABLELOG, |maybe_cache| {
            if maybe_cache.is_none() {
                return None;
            }
            todo!()
        });
        if value.is_none() {
            let index = self.0.get_index();
            let rtx = index.begin_read()?;
            let table = rtx.0.open_table(BRC20_TRANSFERABLELOG)?;
            return get_transferable_by_id(&table, script, inscription_id);
        }
        todo!()
    }

    fn get_inscribe_transfer_inscription(&self, inscription_id: &InscriptionId) -> crate::Result<Option<TransferInfo>, Self::Error> {
        let value: Option<TransferInfo> = self.0.use_cache(CacheTableIndex::BRC20_INSCRIBE_TRANSFER, |maybe_cache| {
            if maybe_cache.is_none() {
                return None;
            }
            todo!()
        });
        if value.is_none() {
            let index = self.0.get_index();
            let rtx = index.begin_read()?;
            let table = rtx.0.open_table(BRC20_INSCRIBE_TRANSFER)?;
            return get_inscribe_transfer_inscription(&table, inscription_id);
        }
        todo!()
    }
}

impl Brc20ReaderWriter for BRC20CacheWrapper {
    fn update_token_balance(&mut self, script_key: &ScriptKey, new_balance: Balance) -> crate::Result<(), Self::Error> {
        let binding = script_tick_key(script_key, &new_balance.tick);
        let key = binding.as_str();
        let binding = rmp_serde::to_vec(&new_balance).unwrap();
        let value = binding.as_slice();
        let key = string_to_bytes(key);
        self.0.use_cache_mut(CacheTableIndex::BRC20_TOKEN, |v| {
            v.data.insert(key, value.to_vec());
        });
        Ok(())
    }

    fn insert_token_info(&mut self, tick: &Tick, new_info: &TokenInfo) -> crate::Result<(), Self::Error> {
        let binding = tick.to_lowercase().hex();
        let key = binding.as_str();
        let binding = rmp_serde::to_vec(new_info).unwrap();
        let value = binding.as_slice();
        self.0.use_cache_mut(CacheTableIndex::BRC20_TOKEN, |v| {
            v.data.insert(key.as_bytes().to_vec(), value.to_vec());
        });
        Ok(())
    }

    fn update_mint_token_info(&mut self, tick: &Tick, minted_amt: u128, minted_block_number: u32) -> crate::Result<(), Self::Error> {
        let mut info = self
            .get_token_info(tick)?
            .unwrap_or_else(|| panic!("token {} not exist", tick.as_str()));

        info.minted = minted_amt;
        info.latest_mint_number = minted_block_number;
        self.0.use_cache_mut(CacheTableIndex::BRC20_TOKEN, |v| {
            let binding = tick.to_lowercase().hex();
            let key = binding.as_str();
            let key = string_to_bytes(key);
            let value = rmp_serde::to_vec(&info).unwrap();
            v.data.insert(key, value.to_vec());
        });
        Ok(())
    }

    fn save_transaction_receipts(&mut self, txid: &Txid, receipt: &[Receipt]) -> crate::Result<(), Self::Error> {
        todo!()
    }

    fn insert_transferable(&mut self, script: &ScriptKey, tick: &Tick, inscription: &TransferableLog) -> crate::Result<(), Self::Error> {
        todo!()
    }

    fn remove_transferable(&mut self, script: &ScriptKey, tick: &Tick, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
        todo!()
    }

    fn insert_inscribe_transfer_inscription(&mut self, inscription_id: &InscriptionId, transfer_info: TransferInfo) -> crate::Result<(), Self::Error> {
        todo!()
    }

    fn remove_inscribe_transfer_inscription(&mut self, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
        todo!()
    }
}