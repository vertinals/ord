use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use anyhow::anyhow;
use bitcoin::Txid;
use log::info;
use {
  super::*,
  crate::{okx::protocol::brc20 as brc20_proto, Result},
};
use crate::index::updater::LATEST_HEIGHT;

pub struct CallManager {}

impl CallManager {
  pub fn new() -> Self {
    Self {}
  }

  pub fn execute_message<T: ContextTrait>(
    &self,
    context: &mut T,
    txid: &Txid,
    msgs: &[Message],
  ) -> Result {
    let mut receipts = vec![];
    // execute message
    for msg in msgs {
      match msg {
        Message::BRC20(brc_msg) => {
          let msg =
            brc20_proto::ExecutionMessage::from_message(context, brc_msg, context.network())?;
          let receipt = brc20_proto::execute(context, &msg)?;
          receipts.push(receipt);
        }
      };
    }


    unsafe {
      if *LATEST_HEIGHT - 1 <=  context.block_height() as u64 {
        // append to file
        write_tx_id_to_file(txid).unwrap();
        info!("save transaction receipts: txid: {}", txid);
      }
    }


    context
      .save_transaction_receipts(txid, &receipts)
      .map_err(|e| anyhow!("failed to add transaction receipt to state! error: {e}"))?;

    Ok(())
  }
}

fn write_tx_id_to_file(txid: &Txid) -> anyhow::Result<()> {
  let path = env::var("TX_IDS_PATH").unwrap_or("/var/txids.txt".to_string());
  let mut file = OpenOptions::new()
      .create(true)
      .append(true)
      .open(path)?;

  let content_to_append = format!("{:?}\n", txid);
  file.write_all(content_to_append.as_bytes())?;
  Ok(())
}
