use crate::types::{GroupType, RdsData};

pub trait RdsDecoderCallbacks {
    fn on_oda(&mut self, app_id: u16, rds_data: &RdsData, group_type: &GroupType);
    fn on_clear(&mut self);
}
