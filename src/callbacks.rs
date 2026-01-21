struct RdsGroup {
    // Fields representing RDS group information
}

struct RdsData {
    // Fields representing decoded RDS data
}

struct RdsBlocks {
    // Fields representing RDS blocks
}

struct RdsGroupType {
    // Fields representing RDS group type
}

/// Trait for objects that want to receive decoded RDS data / events.
pub trait RdsDecoderCallbacks {
    /// Called when a regular (non-ODA) RDS group is successfully decoded.
    fn on_rds_group(&mut self, group: &RdsGroup, data: &RdsData);

    /// Called when an ODA (Open Data Application) block is received.
    ///
    /// This is the equivalent of your C `DecodeODAFunc`.
    fn on_oda(
        &mut self,
        app_id: u16,
        rds_data: &RdsData,
        blocks: &RdsBlocks,
        group_type: RdsGroupType,
        cb_data: Option<&mut ()>, // if you really need cb_data
    );

    // Optional: add more callbacks as needed
    fn on_ps_text_updated(&mut self, ps_text: &str);
    fn on_rt_text_updated(&mut self, rt_text: &str);
}
