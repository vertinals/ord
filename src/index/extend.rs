use super::*;
use redb::ReadTransaction;

impl Index {
  fn get_inscriptions_on_output_with_satpoints_with_rtx(
    outpoint: OutPoint,
    rtx: &ReadTransaction,
  ) -> Result<Vec<(SatPoint, InscriptionId)>> {
    let satpoint_to_sequence_number = rtx.open_multimap_table(SATPOINT_TO_SEQUENCE_NUMBER)?;
    let sequence_number_to_inscription_entry =
      rtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;

    Self::inscriptions_on_output(
      &satpoint_to_sequence_number,
      &sequence_number_to_inscription_entry,
      outpoint,
    )
  }

  pub(crate) fn get_inscriptions_on_output_with_rtx(
    outpoint: OutPoint,
    rtx: &ReadTransaction,
  ) -> Result<Vec<InscriptionId>> {
    Ok(
      Self::get_inscriptions_on_output_with_satpoints_with_rtx(outpoint, rtx)?
        .iter()
        .map(|(_satpoint, inscription_id)| *inscription_id)
        .collect(),
    )
  }

  pub(crate) fn get_inscription_satpoint_by_id_with_rtx(
    inscription_id: InscriptionId,
    rtx: &ReadTransaction,
  ) -> Result<Option<SatPoint>> {
    let Some(sequence_number) = rtx
      .open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?
      .get(&inscription_id.store())?
      .map(|guard| guard.value())
    else {
      return Ok(None);
    };

    let satpoint = rtx
      .open_table(SEQUENCE_NUMBER_TO_SATPOINT)?
      .get(sequence_number)?
      .map(|satpoint| Entry::load(*satpoint.value()));

    Ok(satpoint)
  }
}
