// use std::collections::HashMap;
// use anyhow::anyhow;
// use bitcoin::{BlockHash, OutPoint, Transaction, TxOut};
// use sha3::digest::typenum::op;
// use crate::Inscription;
// use crate::okx::datastore::ord::{Action, DataStoreReadWrite, InscriptionOp};
// use crate::okx::protocol::BlockContext;
//
// pub fn resolve_brczero_inscription<O: DataStoreReadWrite>(
//     context: BlockContext,
//     tx: &Transaction,
//     operations: Vec<InscriptionOp>,
//     blockHash: &BlockHash,
//     ord_store: &O,
// ) -> Result<Vec<BrcZeroMsg>, E> {
//     log::debug!(
//       "Resolve Inscription indexed transaction {}, operations size: {}, data: {:?}",
//       tx.txid(),
//       operations.len(),
//       operations
//     );
//     let mut messages = Vec::new();
//     let mut operation_iter = operations.into_iter().peekable();
//     let new_inscriptions = Inscription::from_transaction(tx)
//         .into_iter()
//         .map(|v| v.inscription)
//         .collect::<Vec<Inscription>>();
//
//     let mut outpoint_to_txout_cache: HashMap<OutPoint, TxOut> = HashMap::new();
//     for input in &tx.input {
//         // "operations" is a list of all the operations in the current block, and they are ordered.
//         // We just need to find the operation corresponding to the current transaction here.
//         while let Some(operation) = operation_iter.peek() {
//             if operation.old_satpoint.outpoint != input.previous_output {
//                 break;
//             }
//             let operation = operation_iter.next().unwrap();
//
//             let sat_in_outputs = operation
//                 .new_satpoint
//                 .map(|satpoint| satpoint.outpoint.txid == operation.txid)
//                 .unwrap_or(false);
//
//             let mut is_transfer = false;
//             let mut sender = "".to_string();
//             let mut inscription_content: String = "".to_string();
//             match operation.action {
//                 // New inscription is not `cursed` or `unbound`.
//                 Action::New {
//                     cursed: false,
//                     unbound: false, ..
//                 } => {
//                     let inscription = new_inscriptions.get(usize::try_from(operation.inscription_id.index).unwrap()).unwrap().clone();
//                     let des_res = deserialize_inscription(&inscription);
//                     match des_res {
//                         Ok(content) => {
//                             let commit_input_satpoint = get_commit_input_satpoint(
//                                 self.client,
//                                 self.state_store.ord(),
//                                 operation.old_satpoint,
//                                 &mut outpoint_to_txout_cache,
//                             )?;
//                             sender = utils::get_script_key_on_satpoint(commit_input_satpoint, self.state_store.ord(), context.network)?.to_string();
//                             self.state_store.ord().save_inscription_with_id(&operation.inscription_id,&inscription).map_err(|e| {
//                                 anyhow!("failed to set inscription with id in ordinals operations to state! error: {e}")
//                             })?;
//                             inscription_content = content;
//                         },
//                         Err(err) => {
//                             continue;
//                         },
//                     }
//                 },
//                 // Transfer inscription operation.
//                 Action::Transfer => {
//                     if operation.inscription_id.txid == operation.old_satpoint.outpoint.txid &&
//                         operation.inscription_id.index == operation.old_satpoint.outpoint.vout {
//                         is_transfer = true;
//                         ord_store.get_transaction_operations()
//                         let inscription = match get_inscription_by_id(self.client,self.state_store.ord(), &operation.inscription_id) {
//                             Ok(innnet_inscription) => {innnet_inscription}
//                             Err(err) => {continue}
//                         };
//                     }
//
//
//                     self.state_store.ord().remove_inscription_with_id(&operation.inscription_id).map_err(|e| {
//                         anyhow!("failed to remove inscription with id in ordinals operations to state! error: {e}")
//                     })?;
//                     let des_res = deserialize_inscription(&inscription);
//                     match des_res {
//                         Ok(content) => {
//                             sender = utils::get_script_key_on_satpoint(operation.old_satpoint, self.state_store.ord(), context.network)?.to_string();
//                             inscription_content = content;
//                         },
//                         Err(err) => {
//                             continue;
//                         },
//                     }
//                 },
//                 _ => {
//                     continue;},
//             };
//             messages.push(BrcZeroMsg{
//                 0,
//                 msg: MsgInscription {
//                     inscription: inscription_content,
//                     inscription_context: InscriptionContext {
//                         txid: operation.txid.to_string(),
//                         inscription_id: operation.inscription_id.to_string(),
//                         inscription_number: utils::get_inscription_number_by_id(operation.inscription_id, self.state_store.ord())?,
//                         old_sat_point: operation.old_satpoint.to_string(),
//                         new_sat_point: operation.new_satpoint.unwrap().to_string(),
//                         sender,
//                         receiver: if sat_in_outputs {
//                             utils::get_script_key_on_satpoint(
//                                 operation.new_satpoint.unwrap(),
//                                 self.state_store.ord(),
//                                 context.network,
//                             )?.to_string()
//                         } else {
//                             "".to_string()
//                         },
//                         is_transfer,
//                         block_height: context.blockheight,
//                         block_time: context.blocktime,
//                         block_hash: blockHash.to_string(),
//                     },
//                 }
//             });
//         }
//     }
//     Ok(messages)
// }
pub(crate) mod zerodata;
pub(crate) mod error;