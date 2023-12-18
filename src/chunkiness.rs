use super::*;

pub(crate) enum Chunkiness {
  Fine,
  Sand,
  Peebles,
  Rocks,
  Chongus,
}

impl From<f64> for Chunkiness {
  fn from(chunk_factor: f64) -> Self {
    match chunk_factor {
      f if f >= 1.0 && f < 2.0 => Self::Fine,
      f if f >= 2.0 && f < 3.0 => Self::Sand,
      f if f >= 3.0 && f < 4.0 => Self::Peebles,
      f if f >= 4.0 && f < 5.0 => Self::Rocks,
      _ => Self::Chongus,
    }
  }
}

impl Chunkiness {
  pub(crate) fn calculate_factor(divisibility: u64, supply: u64) -> f64 {
    // (((divisibility + 1) * supply) as f64).log10()

    let unit_count = supply.saturating_mul(divisibility + 1);

    unit_count
  }
}

#[cfg(test)]
mod tests {
  use crate::runes::MAX_DIVISIBILITY;

  use super::*;

  #[test]
  fn chunkiness() {
    println!(
      "Fine: {}",
      Chunkiness::calculate_factor(MAX_DIVISIBILITY.into(), u64::MAX)
    );

    println!("Bitcoin: {}", Chunkiness::calculate_factor(8, 21_000_000));

    println!("Chunkiest: {}", Chunkiness::calculate_factor(0, 1));
  }
}
