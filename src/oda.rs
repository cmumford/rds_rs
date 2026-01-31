use crate::rds::RdsData;
use crate::types::{Group, GroupType, ValidFields};
use heapless::LinearMap;

const INVALID_ODA_APP_ID: u16 = 0x0;

#[derive(Default, Clone, PartialEq, Eq)]
pub struct OdaEntry {
    pub group_type: GroupType,
    pub packet_count: u16,
}

/// Is the ODA application ID valid?
pub fn is_valid_oda_app_id(app_id: u16) -> bool {
    return app_id != INVALID_ODA_APP_ID;
}

pub fn is_oda_group_type_used(map: &LinearMap<u16, OdaEntry, 10>, gt: GroupType) -> bool {
    for (_key, val) in map.iter() {
        if val.group_type == gt {
            return true;
        }
    }
    return false;
}

pub fn decode_oda(_group: &Group, gt: GroupType, rds_data: &mut RdsData) -> ValidFields {
    let mut app_id: u16 = INVALID_ODA_APP_ID;
    for (key, val) in rds_data.oda.iter() {
        if val.group_type == gt {
            app_id = *key;
            break;
        }
    }
    if app_id == INVALID_ODA_APP_ID {
        return ValidFields::new();
    }
    // TODO: Finish this. Either use callback, or another way for caller to know new ODA has arrived.
    ValidFields::new()
}
