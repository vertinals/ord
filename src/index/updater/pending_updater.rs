use {
    super::*,
    crate::okx::datastore::ord::operation::{Action, InscriptionOp},
};
use crate::index::updater::inscription_updater::{Flotsam, Origin};
use crate::okx::datastore::cache::{CacheTableIndex, CacheWriter};


pub struct PendingStorageProcessor<'a, 'db, 'tx> {
    // cache_writer: CacheWriter,
    pub(super) home_inscriptions: &'a mut Table<'db, 'tx, u32, InscriptionIdValue>,
    pub(super) id_to_sequence_number: &'a mut Table<'db, 'tx, InscriptionIdValue, u32>,
    pub(super) inscription_number_to_sequence_number: &'a mut Table<'db, 'tx, i32, u32>,
    pub(super) outpoint_to_entry: &'a mut Table<'db, 'tx, &'static OutPointValue, &'static [u8]>,
    pub(super) transaction_id_to_transaction:
    &'a mut Table<'db, 'tx, &'static TxidValue, &'static [u8]>,
    pub(super) sat_to_sequence_number: &'a mut MultimapTable<'db, 'tx, u64, u32>,
    pub(super) satpoint_to_sequence_number:
    &'a mut MultimapTable<'db, 'tx, &'static SatPointValue, u32>,
    pub(super) sequence_number_to_children: &'a mut MultimapTable<'db, 'tx, u32, u32>,
    pub(super) sequence_number_to_entry: &'a mut Table<'db, 'tx, u32, InscriptionEntryValue>,
    pub(super) sequence_number_to_satpoint: &'a mut Table<'db, 'tx, u32, &'static SatPointValue>,
}


impl<'a, 'db, 'tx> PendingStorageProcessor<'a, 'db, 'tx> {
    pub(crate) fn satpoint_to_sequence_number_remove_all(&mut self, v: &SatPointValue) -> crate::Result<()> {
        self
            .satpoint_to_sequence_number
            .remove_all(v)?;
        Ok(())
    }
    pub fn new(home_inscriptions: &'a mut Table<'db, 'tx, u32, InscriptionIdValue>, id_to_sequence_number: &'a mut Table<'db, 'tx, InscriptionIdValue, u32>, inscription_number_to_sequence_number: &'a mut Table<'db, 'tx, i32, u32>, outpoint_to_entry: &'a mut Table<'db, 'tx, &'static OutPointValue, &'static [u8]>, transaction_id_to_transaction: &'a mut Table<'db, 'tx, &'static TxidValue, &'static [u8]>, sat_to_sequence_number: &'a mut MultimapTable<'db, 'tx, u64, u32>, satpoint_to_sequence_number: &'a mut MultimapTable<'db, 'tx, &'static SatPointValue, u32>, sequence_number_to_children: &'a mut MultimapTable<'db, 'tx, u32, u32>, sequence_number_to_entry: &'a mut Table<'db, 'tx, u32, InscriptionEntryValue>, sequence_number_to_satpoint: &'a mut Table<'db, 'tx, u32, &'static SatPointValue>) -> Self {
        Self { home_inscriptions, id_to_sequence_number, inscription_number_to_sequence_number, outpoint_to_entry, transaction_id_to_transaction, sat_to_sequence_number, satpoint_to_sequence_number, sequence_number_to_children, sequence_number_to_entry, sequence_number_to_satpoint }
    }
}

impl<'a, 'db, 'tx> PendingStorageProcessor<'a, 'db, 'tx> {
    pub(crate) fn home_inscriptions_len(&self) -> u64 {
        todo!()
    }
}


impl<'a, 'db, 'tx> PendingStorageProcessor<'a, 'db, 'tx> {
    pub(crate) fn sequence_number_to_satpoint_insert(&mut self, sequence_number: u32, sat_point: &SatPointValue) -> crate::Result<()> {
        self.sequence_number_to_satpoint.insert(sequence_number, sat_point)?;
        // let key = sequence_number.to_le_bytes().as_slice();
        // let value = rmp_serde::to_vec(sat_point).unwrap();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SEQUENCE_NUMBER_TO_SATPOINT, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        Ok(())
    }
    pub(crate) fn satpoint_to_sequence_number_insert(&mut self, sat_point: &SatPointValue, sequence: u32) -> crate::Result<()> {
        self.sequence_number_to_satpoint.insert(sequence, sat_point)?;
        // let key = rmp_serde::to_vec(sat_point).unwrap();
        // let value = sequence.to_le_bytes().as_slice();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SAT_TO_SEQUENCE_NUMBER, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        Ok(())
    }
    pub(crate) fn home_inscriptions_pop_first(&mut self) -> crate::Result<()> {
        self.home_inscriptions.pop_first()?;
        // self.cache_writer.use_cache_mut(CacheTableIndex::HOME_INSCRIPTIONS, |v| {
        //     v.pop_first()
        // });
        // Ok(())
        Ok(())
    }
    pub(crate) fn home_inscriptions_insert(&mut self, sequence_number: &u32, value: InscriptionIdValue) -> crate::Result<()> {
        // let key = sequnce_number.to_le_bytes().as_slice();
        // let value = rmp_serde::to_vec(&value).unwrap();
        // self.cache_writer.use_cache_mut(CacheTableIndex::HOME_INSCRIPTIONS, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        self
            .home_inscriptions
            .insert(sequence_number, value)?;
        Ok(())
    }
    pub(crate) fn id_to_sequence_number_insert(&mut self, value: &InscriptionIdValue, sequence_number: u32) -> crate::Result<()> {
        // let key = rmp_serde::to_vec(value).unwrap();
        // let value = sequence.to_le_bytes().as_slice();
        // self.cache_writer.use_cache_mut(CacheTableIndex::INSCRIPTION_ID_TO_SEQUENCE_NUMBER, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        self
            .id_to_sequence_number
            .insert(value, sequence_number)?;
        Ok(())
    }
    pub(crate) fn sequence_number_to_children_insert(&mut self, parent_sequence_number: u32, sequence_number: u32) -> crate::Result<()> {
        // let key = sequence.to_le_bytes().as_slice();
        // let value = rmp_serde::to_vec(value).unwrap();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SEQUENCE_NUMBER_TO_CHILDREN, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        self
            .sequence_number_to_children
            .insert(parent_sequence_number, sequence_number)?;
        Ok(())
    }
    pub(crate) fn sequence_number_to_entry_insert(&mut self, sequence: u32, value: &InscriptionEntryValue) -> crate::Result<()> {
        // let key = sequence.to_le_bytes().as_slice();
        // let value = rmp_serde::to_vec(value).unwrap();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        self.sequence_number_to_entry.insert(sequence, value)?;
        Ok(())
    }
    pub(crate) fn sat_to_sequence_number_insert(&mut self, n: &u64, sequence_number: &u32) -> crate::Result<()> {
        // let key = n.to_le_bytes().as_slice();
        // let value = sequence.to_le_bytes().as_slice();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SAT_TO_SEQUENCE_NUMBER, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        self.sat_to_sequence_number.insert(n, sequence_number)?;
        Ok(())
    }
    pub(crate) fn inscription_number_to_sequence_number_insert(&mut self, inscription_number: i32, sequence_number: u32) -> crate::Result<()> {
        // let key = inscription_number.to_le_bytes().as_slice();
        // let value = sequence_number.to_le_bytes().as_slice();
        // self.cache_writer.use_cache_mut(CacheTableIndex::INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        self
            .inscription_number_to_sequence_number
            .insert(inscription_number, sequence_number)?;
        Ok(())
    }
    pub(crate) fn outpoint_to_entry_insert(&mut self, value: &OutPointValue, entry: &[u8]) -> crate::Result<()> {
        self.outpoint_to_entry.insert(value, entry)?;
        Ok(())
        // let key = rmp_serde::to_vec(value).unwrap();
        // let value = entry.to_vec();
        // self.cache_writer.use_cache_mut(CacheTableIndex::OUTPOINT_TO_ENTRY, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
    }
    pub fn inscriptions_on_output(&mut self, prev_output: &OutPoint) -> crate::Result<Vec<(SatPoint, InscriptionId)>> {
        let ret = Index::inscriptions_on_output(
            self.satpoint_to_sequence_number,
            self.sequence_number_to_entry,
            prev_output.clone())?;
        // TODO: twice
        todo!()
    }

    pub(crate) fn transaction_id_to_transaction_insert(&mut self, tx_id: &TxidValue, value: &[u8]) -> crate::Result<()> {
        self
            .transaction_id_to_transaction
            .insert(tx_id, value)?;

        Ok(())
    }

    pub(crate) fn id_to_sequence_number_get(&self, x: InscriptionIdValue) -> crate::Result<Option<u32>> {
        // TODO,twice
        let ret = self.id_to_sequence_number.get(x)?.unwrap().value();
        Ok(Some(ret))
    }
    pub fn sequence_number_to_entry_get(&self, initial_inscription_sequence_number: u32) -> crate::Result<Option<InscriptionEntryValue>> {
        // TODO: twice
        let ret = self
            .sequence_number_to_entry
            .get(initial_inscription_sequence_number)?
            .unwrap()
            .value();
        Ok(Some(ret))
    }
}

pub struct PendingUpdater<'a, 'db, 'tx> {
    processor: &'a mut PendingStorageProcessor<'a, 'db, 'tx>,
    pub(super) operations: &'a mut HashMap<Txid, Vec<InscriptionOp>>,
    pub(super) blessed_inscription_count: u64,
    pub(super) chain: Chain,
    pub(super) cursed_inscription_count: u64,
    pub(super) flotsam: Vec<Flotsam>,
    pub(super) height: u32,
    pub(super) home_inscription_count: u64,
    pub(super) index_transactions: bool,
    pub(super) next_sequence_number: u32,
    pub(super) lost_sats: u64,
    pub(super) reward: u64,
    pub(super) transaction_buffer: Vec<u8>,
    pub(super) timestamp: u32,
    pub(super) unbound_inscriptions: u64,
    pub(super) tx_out_receiver: &'a mut Receiver<TxOut>,
    pub(super) tx_out_cache: &'a mut SimpleLru<OutPoint, TxOut>,
    pub(super) new_outpoints: Vec<OutPoint>,
}

impl<'a, 'db, 'tx> PendingUpdater<'a, 'db, 'tx> {
    pub fn new(
        operations: &'a mut HashMap<Txid, Vec<InscriptionOp>>,
        blessed_inscription_count: u64,
        chain: Chain,
        cursed_inscription_count: u64,
        height: u32,
        index_transactions: bool,
        next_sequence_number: u32,
        lost_sats: u64,
        timestamp: u32,
        unbound_inscriptions: u64,
        tx_out_receiver: &'a mut Receiver<TxOut>,
        tx_out_cache: &'a mut SimpleLru<OutPoint, TxOut>,
        processor: &'a mut PendingStorageProcessor<'a, 'db, 'tx>,
    ) -> Result<Self> {
        let home_inscriptions_len = processor.home_inscriptions_len();
        Ok(Self {
            processor,
            operations,
            blessed_inscription_count,
            chain,
            cursed_inscription_count,
            flotsam: vec![],
            height,
            home_inscription_count: home_inscriptions_len,
            index_transactions,
            next_sequence_number,
            lost_sats,
            reward: Height(height).subsidy(),
            transaction_buffer: vec![],
            timestamp,
            unbound_inscriptions,
            tx_out_receiver,
            tx_out_cache,
            new_outpoints: vec![],
        })
    }
    pub fn index_envelopes(
        &mut self,
        tx: &Transaction,
        txid: Txid,
        input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
    ) -> Result {
        let mut floating_inscriptions = Vec::new();
        let mut id_counter = 0;
        let mut inscribed_offsets = BTreeMap::new();
        let jubilant = self.height >= self.chain.jubilee_height();
        let mut total_input_value = 0;
        let total_output_value = tx.output.iter().map(|txout| txout.value).sum::<u64>();

        let envelopes = ParsedEnvelope::from_transaction(tx);
        let inscriptions = !envelopes.is_empty();
        let mut envelopes = envelopes.into_iter().peekable();

        for (input_index, tx_in) in tx.input.iter().enumerate() {
            // skip subsidy since no inscriptions possible
            if tx_in.previous_output.is_null() {
                total_input_value += Height(self.height).subsidy();
                continue;
            }


            // find existing inscriptions on input (transfers of inscriptions)
            for (old_satpoint, inscription_id) in self.processor.inscriptions_on_output(&tx_in.previous_output)? {
                let offset = total_input_value + old_satpoint.offset;
                floating_inscriptions.push(Flotsam {
                    txid,
                    offset,
                    inscription_id,
                    old_satpoint,
                    origin: Origin::Old,
                });

                inscribed_offsets
                    .entry(offset)
                    .or_insert((inscription_id, 0))
                    .1 += 1;
            }

            let offset = total_input_value;

            // multi-level cache for UTXO set to get to the input amount
            let current_input_value = if let Some(tx_out) = self.tx_out_cache.get(&tx_in.previous_output)
            {
                tx_out.value
            } else {
                let tx_out = self.tx_out_receiver.blocking_recv().ok_or_else(|| {
                    anyhow!(
            "failed to get transaction for {}",
            tx_in.previous_output.txid
          )
                })?;
                // received new tx out from chain node, add it to new_outpoints first and persist it in db later.
                #[cfg(not(feature = "cache"))]
                self.new_outpoints.push(tx_in.previous_output);
                self
                    .tx_out_cache
                    .insert(tx_in.previous_output, tx_out.clone());
                tx_out.value
            };

            total_input_value += current_input_value;

            // go through all inscriptions in this input
            while let Some(inscription) = envelopes.peek() {
                if inscription.input != u32::try_from(input_index).unwrap() {
                    break;
                }

                let inscription_id = InscriptionId {
                    txid,
                    index: id_counter,
                };

                let curse = if inscription.payload.unrecognized_even_field {
                    Some(crate::index::updater::inscription_updater::Curse::UnrecognizedEvenField)
                } else if inscription.payload.duplicate_field {
                    Some(crate::index::updater::inscription_updater::Curse::DuplicateField)
                } else if inscription.payload.incomplete_field {
                    Some(crate::index::updater::inscription_updater::Curse::IncompleteField)
                } else if inscription.input != 0 {
                    Some(crate::index::updater::inscription_updater::Curse::NotInFirstInput)
                } else if inscription.offset != 0 {
                    Some(crate::index::updater::inscription_updater::Curse::NotAtOffsetZero)
                } else if inscription.payload.pointer.is_some() {
                    Some(crate::index::updater::inscription_updater::Curse::Pointer)
                } else if inscription.pushnum {
                    Some(crate::index::updater::inscription_updater::Curse::Pushnum)
                } else if inscription.stutter {
                    Some(crate::index::updater::inscription_updater::Curse::Stutter)
                } else if let Some((id, count)) = inscribed_offsets.get(&offset) {
                    if *count > 1 {
                        Some(crate::index::updater::inscription_updater::Curse::Reinscription)
                    } else {
                        let initial_inscription_sequence_number =
                            self.processor.id_to_sequence_number_get(id.store())?.unwrap();


                        let entry = InscriptionEntry::load(self.processor.sequence_number_to_entry_get(initial_inscription_sequence_number)?.unwrap());

                        let initial_inscription_was_cursed_or_vindicated =
                            entry.inscription_number < 0 || Charm::Vindicated.is_set(entry.charms);

                        if initial_inscription_was_cursed_or_vindicated {
                            None
                        } else {
                            Some(crate::index::updater::inscription_updater::Curse::Reinscription)
                        }
                    }
                } else {
                    None
                };

                let unbound = current_input_value == 0
                    || curse == Some(crate::index::updater::inscription_updater::Curse::UnrecognizedEvenField)
                    || inscription.payload.unrecognized_even_field;

                let offset = inscription
                    .payload
                    .pointer()
                    .filter(|&pointer| pointer < total_output_value)
                    .unwrap_or(offset);

                floating_inscriptions.push(Flotsam {
                    txid,
                    inscription_id,
                    offset,
                    old_satpoint: SatPoint {
                        outpoint: tx_in.previous_output,
                        offset: 0,
                    },
                    origin: Origin::New {
                        cursed: curse.is_some() && !jubilant,
                        fee: 0,
                        hidden: inscription.payload.hidden(),
                        parent: inscription.payload.parent(),
                        pointer: inscription.payload.pointer(),
                        reinscription: inscribed_offsets.get(&offset).is_some(),
                        unbound,
                        inscription: inscription.payload.clone(),
                        vindicated: curse.is_some() && jubilant,
                    },
                });

                inscribed_offsets
                    .entry(offset)
                    .or_insert((inscription_id, 0))
                    .1 += 1;

                envelopes.next();
                id_counter += 1;
            }
        }


        if self.index_transactions && inscriptions {
            tx.consensus_encode(&mut self.transaction_buffer)
                .expect("in-memory writers don't error");
            self.processor.transaction_id_to_transaction_insert(&txid.store(), self.transaction_buffer.as_slice())?;

            self.transaction_buffer.clear();
        }

        let potential_parents = floating_inscriptions
            .iter()
            .map(|flotsam| flotsam.inscription_id)
            .collect::<HashSet<InscriptionId>>();

        for flotsam in &mut floating_inscriptions {
            if let Flotsam {
                origin: Origin::New { parent, .. },
                ..
            } = flotsam
            {
                if let Some(purported_parent) = parent {
                    if !potential_parents.contains(purported_parent) {
                        *parent = None;
                    }
                }
            }
        }

        // still have to normalize over inscription size
        for flotsam in &mut floating_inscriptions {
            if let Flotsam {
                origin: Origin::New { ref mut fee, .. },
                ..
            } = flotsam
            {
                *fee = (total_input_value - total_output_value) / u64::from(id_counter);
            }
        }

        let is_coinbase = tx
            .input
            .first()
            .map(|tx_in| tx_in.previous_output.is_null())
            .unwrap_or_default();

        if is_coinbase {
            floating_inscriptions.append(&mut self.flotsam);
        }

        floating_inscriptions.sort_by_key(|flotsam| flotsam.offset);
        let mut inscriptions = floating_inscriptions.into_iter().peekable();

        let mut range_to_vout = BTreeMap::new();
        let mut new_locations = Vec::new();
        let mut output_value = 0;
        for (vout, tx_out) in tx.output.iter().enumerate() {
            let end = output_value + tx_out.value;

            while let Some(flotsam) = inscriptions.peek() {
                if flotsam.offset >= end {
                    break;
                }

                let new_satpoint = SatPoint {
                    outpoint: OutPoint {
                        txid,
                        vout: vout.try_into().unwrap(),
                    },
                    offset: flotsam.offset - output_value,
                };

                new_locations.push((new_satpoint, inscriptions.next().unwrap()));
            }

            range_to_vout.insert((output_value, end), vout.try_into().unwrap());

            output_value = end;

            #[cfg(not(feature = "cache"))]
            self.new_outpoints.push(OutPoint {
                vout: vout.try_into().unwrap(),
                txid,
            });
            self.tx_out_cache.insert(
                OutPoint {
                    vout: vout.try_into().unwrap(),
                    txid,
                },
                tx_out.clone(),
            );
        }

        for (new_satpoint, mut flotsam) in new_locations.into_iter() {
            let new_satpoint = match flotsam.origin {
                Origin::New {
                    pointer: Some(pointer),
                    ..
                } if pointer < output_value => {
                    match range_to_vout.iter().find_map(|((start, end), vout)| {
                        (pointer >= *start && pointer < *end).then(|| (vout, pointer - start))
                    }) {
                        Some((vout, offset)) => {
                            flotsam.offset = pointer;
                            SatPoint {
                                outpoint: OutPoint { txid, vout: *vout },
                                offset,
                            }
                        }
                        _ => new_satpoint,
                    }
                }
                _ => new_satpoint,
            };

            self.update_inscription_location(input_sat_ranges, flotsam, new_satpoint)?;
        }

        if is_coinbase {
            for flotsam in inscriptions {
                let new_satpoint = SatPoint {
                    outpoint: OutPoint::null(),
                    offset: self.lost_sats + flotsam.offset - output_value,
                };
                self.update_inscription_location(input_sat_ranges, flotsam, new_satpoint)?;
            }
            self.lost_sats += self.reward - output_value;
            Ok(())
        } else {
            self.flotsam.extend(inscriptions.map(|flotsam| Flotsam {
                offset: self.reward + flotsam.offset - output_value,
                ..flotsam
            }));
            self.reward += total_input_value - output_value;
            Ok(())
        }
    }

    // write tx_out to outpoint_to_entry table
    pub fn flush_cache(self) -> Result {
        let start = Instant::now();
        let persist = self.new_outpoints.len();
        let mut entry = Vec::new();
        for outpoint in self.new_outpoints.into_iter() {
            let tx_out = self.tx_out_cache.get(&outpoint).unwrap();
            tx_out.consensus_encode(&mut entry)?;
            self.processor.outpoint_to_entry_insert(&outpoint.store(), entry.as_slice())?;
            entry.clear();
        }
        log::info!(
      "flush cache, persist:{}, global:{} cost: {}ms",
      persist,
      self.tx_out_cache.len(),
      start.elapsed().as_millis()
    );
        Ok(())
    }

    fn calculate_sat(
        input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
        input_offset: u64,
    ) -> Option<Sat> {
        let input_sat_ranges = input_sat_ranges?;

        let mut offset = 0;
        for (start, end) in input_sat_ranges {
            let size = end - start;
            if offset + size > input_offset {
                let n = start + input_offset - offset;
                return Some(Sat(n));
            }
            offset += size;
        }

        unreachable!()
    }

    fn update_inscription_location(
        &mut self,
        input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
        flotsam: Flotsam,
        new_satpoint: SatPoint,
    ) -> Result {
        let inscription_id = flotsam.inscription_id;
        let (unbound, sequence_number) = match flotsam.origin {
            Origin::Old => {
                self
                    .processor.satpoint_to_sequence_number_remove_all(&flotsam.old_satpoint.store())?;

                (
                    false,
                    self
                        .processor.id_to_sequence_number_get(
                        inscription_id.store())?
                        .unwrap()
                )
            }
            Origin::New {
                cursed,
                fee,
                hidden,
                parent,
                pointer: _,
                reinscription,
                unbound,
                inscription: _,
                vindicated,
            } => {
                let inscription_number = if cursed {
                    let number: i32 = self.cursed_inscription_count.try_into().unwrap();
                    self.cursed_inscription_count += 1;

                    // because cursed numbers start at -1
                    -(number + 1)
                } else {
                    let number: i32 = self.blessed_inscription_count.try_into().unwrap();
                    self.blessed_inscription_count += 1;

                    number
                };

                let sequence_number = self.next_sequence_number;
                self.next_sequence_number += 1;

                self
                    .processor.inscription_number_to_sequence_number_insert(
                    inscription_number, sequence_number)?;

                let sat = if unbound {
                    None
                } else {
                    Self::calculate_sat(input_sat_ranges, flotsam.offset)
                };

                let mut charms = 0;

                if cursed {
                    Charm::Cursed.set(&mut charms);
                }

                if reinscription {
                    Charm::Reinscription.set(&mut charms);
                }

                if let Some(sat) = sat {
                    if sat.nineball() {
                        Charm::Nineball.set(&mut charms);
                    }

                    if sat.coin() {
                        Charm::Coin.set(&mut charms);
                    }

                    match sat.rarity() {
                        Rarity::Common | Rarity::Mythic => {}
                        Rarity::Uncommon => Charm::Uncommon.set(&mut charms),
                        Rarity::Rare => Charm::Rare.set(&mut charms),
                        Rarity::Epic => Charm::Epic.set(&mut charms),
                        Rarity::Legendary => Charm::Legendary.set(&mut charms),
                    }
                }

                if new_satpoint.outpoint == OutPoint::null() {
                    Charm::Lost.set(&mut charms);
                }

                if unbound {
                    Charm::Unbound.set(&mut charms);
                }

                if vindicated {
                    Charm::Vindicated.set(&mut charms);
                }

                if let Some(Sat(n)) = sat {
                    self.processor.sat_to_sequence_number_insert(&n, &sequence_number)?;
                }

                let parent = match parent {
                    Some(parent_id) => {
                        let parent_sequence_number = self
                            .processor.id_to_sequence_number_get(
                            parent_id.store())?
                            .unwrap();
                        self
                            .processor.sequence_number_to_children_insert(
                            parent_sequence_number, sequence_number)?;

                        Some(parent_sequence_number)
                    }
                    None => None,
                };

                self.processor.sequence_number_to_entry_insert(
                    sequence_number,
                    &InscriptionEntry {
                        charms,
                        fee,
                        height: self.height,
                        id: inscription_id,
                        inscription_number,
                        parent,
                        sat,
                        sequence_number,
                        timestamp: self.timestamp,
                    }
                        .store(),
                )?;

                self
                    .processor.id_to_sequence_number_insert(
                    &inscription_id.store(), sequence_number)?;

                if !hidden {
                    self
                        .processor.home_inscriptions_insert(
                        &sequence_number, inscription_id.store())?;

                    if self.home_inscription_count == 100 {
                        self.processor.home_inscriptions_pop_first()?;
                    } else {
                        self.home_inscription_count += 1;
                    }
                }

                (unbound, sequence_number)
            }
        };

        let satpoint = if unbound {
            let new_unbound_satpoint = SatPoint {
                outpoint: unbound_outpoint(),
                offset: self.unbound_inscriptions,
            };
            self.unbound_inscriptions += 1;
            new_unbound_satpoint.store()
        } else {
            new_satpoint.store()
        };

        self
            .operations
            .entry(flotsam.txid)
            .or_default()
            .push(InscriptionOp {
                txid: flotsam.txid,
                sequence_number,
                inscription_number: self
                    .processor.sequence_number_to_entry_get(
                    sequence_number)?
                    .map(|entry| InscriptionEntry::load(entry).inscription_number),
                inscription_id: flotsam.inscription_id,
                action: match flotsam.origin {
                    Origin::Old => Action::Transfer,
                    Origin::New {
                        cursed,
                        fee: _,
                        hidden: _,
                        parent: _,
                        pointer: _,
                        reinscription: _,
                        unbound,
                        inscription,
                        vindicated,
                    } => Action::New {
                        cursed,
                        unbound,
                        vindicated,
                        inscription,
                    },
                },
                old_satpoint: flotsam.old_satpoint,
                new_satpoint: Some(Entry::load(satpoint)),
            });

        self
            .processor.satpoint_to_sequence_number_insert(
            &satpoint, sequence_number)?;
        self
            .processor.sequence_number_to_satpoint_insert(
            sequence_number, &satpoint)?;

        Ok(())
    }
}

#[test]
pub fn test_asd() {}