pub struct RdsGroup {
    // Fields representing RDS group information
}

pub struct RdsData {
    // Fields representing decoded RDS data
}

pub struct RdsGroupType {
    // Fields representing RDS group type
}

/// Trait for objects that want to receive decoded RDS data / events.
pub trait RdsDecoderCallbacks {
    /// Called when a regular (non-ODA) RDS group is successfully decoded.
    fn on_rds_group(&mut self, group: &RdsGroup, data: &RdsData);

    /// Called when an ODA (Open Data Application) block is received.
    fn on_oda(
        &mut self,
        app_id: u16,
        rds_data: &RdsData,
        group_type: RdsGroupType,
        cb_data: Option<&mut ()>, // if you really need cb_data
    );
}
