use std::env;
use std::fs::read_to_string;
use reqwest::blocking;

use serde::{Deserialize, Serialize};

use ord::Receipt;

fn main() {
    // read from stdio
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("please provide enough parameters")
    }

    // first arg
    let tx_id_file = &args[1];
    let rpc_url = &args[2];

    let data = read_to_string(tx_id_file).unwrap();

    let tx_ids: Vec<&str> = data.split('\n').collect();
    let mut hit = 0;
    let mut total = tx_ids.len();
    for id in tx_ids {
        if id.len() != 66 {
            println!("{} is invalid tx_id", id);
            total -= 1;
            continue
        }
        match check(&id[2..], rpc_url) {
            Ok(ret) => {
                if ret {
                    hit += 1;
                }else {
                    println!("{} is different",id)
                }
            }
            Err(_) => {}
        }

    }

    let percent = hit as f32 / total as f32 * 100f32;

    println!("hit ratio: {:.2} %",percent)
}

fn check(tx_id: &str, url: &String) -> anyhow::Result<bool> {
    let url = format!("{}/{}", url, tx_id);
    let response = blocking::get(url)?;

    if response.status().is_success() {
        let receipt = response.json::<MultipleReceipt>().unwrap();
        if receipt.confirm.len() != receipt.pending.len() {
            return Ok(false);
        }

        if receipt.confirm.len() > 0 {
            let receip1 = &receipt.confirm[0];
            let receip2 = &receipt.pending[0];
            return Ok(equal(receip1, receip2));
        }

        return Ok(true);
    }
    return Ok(false);
}


#[derive(Serialize, Deserialize)]
pub struct MultipleReceipt {
    pub confirm: Vec<Receipt>,
    pub pending: Vec<Receipt>,
}

fn equal(receipt: &Receipt, receipt2: &Receipt) -> bool {
    return receipt.old_satpoint == receipt2.old_satpoint &&
        receipt.new_satpoint == receipt2.new_satpoint &&
        receipt.op == receipt2.op &&
        receipt.from == receipt2.from &&
        receipt.to == receipt2.to &&
        receipt.result == receipt2.result;
}