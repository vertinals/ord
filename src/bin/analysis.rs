use std::env;
use std::fs::{OpenOptions, read_to_string};
use std::io::Write;
use bitcoin::Txid;
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

    let tx_ids: Vec<&str> = data.split('\n').filter(|x|x.len() > 0).collect();
    let mut hit = 0;
    let mut right = 0;
    let mut sum = 0;
    let mut total = tx_ids.len();
    for id in tx_ids {
        if id.len() != 66 {
            println!("{} is invalid tx_id", id);
            total -= 1;
            continue;
        }
        match check(&id[2..], rpc_url) {
            Ok(ret) => {
                sum += 1;
                if ret.0 {
                    right += 1;
                    println!("{}",id)
                } else {
                    write_tx_id_to_file(id);
                }

                if ret.1 {
                    hit += 1
                }
            }
            Err(_) => {}
        }
    }

    let percent = hit as f32 / total as f32 * 100f32;

    println!("hit ratio: {:.2} %", percent);

    let correct_percebt = right as f32 /  hit as f32 * 100f32;

    println!("correct ratio: {:.2} %", correct_percebt);
}

fn check(tx_id: &str, url: &String) -> anyhow::Result<(bool, bool)> {
    let url = format!("{}/{}", url, tx_id);
    let response = blocking::get(url)?;

    if response.status().is_success() {
        let receipt = response.json::<MultipleReceipt>().unwrap();
        if receipt.confirm.len() == 0 {
            println!("{} confirm empty", tx_id);
            return Ok((false, false));
        }
        if receipt.pending.len() == 0 {
            println!("{} pending empty", tx_id);
            return Ok((false, false));
        }

        println!("{} confirm len: {}, pending len: {}", tx_id, receipt.confirm.len(), receipt.pending.len());
        if receipt.confirm.len() < receipt.pending.len() {
            println!("{} right", tx_id);
            return Ok((true, true));
        }


        let receip1 = &receipt.confirm[0];
        let receip2 = &receipt.pending[0];
        let ret = equal(receip1, receip2);
        if ret {
            println!("{} right", tx_id);
        }
        return Ok((ret, true));
    }
    println!("{} query error", tx_id);
    return Ok((false,false));
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

fn write_tx_id_to_file(txid: &str) -> anyhow::Result<()> {
    let path = "err_txids.txt";
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    let content_to_append = format!("{}\n", txid);
    file.write_all(content_to_append.as_bytes())?;
    Ok(())
}