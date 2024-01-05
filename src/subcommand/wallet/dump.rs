use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Dump {
  #[arg(
    long,
    default_value = "",
    help = "Use <ADDRESS> to dump the private key."
  )]
  pub(crate) address: String,
}

impl Dump {
  pub(crate) fn run(self, wallet: String, options: Options) -> SubcommandResult {
    let address = bitcoin::Address::from_str(&self.address)?;
    let address = address.require_network(Network::Testnet)?;
    let priv_key =
      bitcoin_rpc_client_for_wallet_command(wallet, &options)?.dump_private_key(&address)?;

    Ok(Box::new(priv_key))
  }
}
