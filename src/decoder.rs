use crate::callbacks::RdsDecoderCallbacks;
use crate::types::{GroupType, RdsData};

pub struct RdsBlocks {
    pub a: Option<u16>,
    pub b: Option<u16>,
    pub c: Option<u16>,
    pub d: Option<u16>,
}

pub struct Decoder<'a> {
    callbacks: &'a mut dyn RdsDecoderCallbacks,
}

impl<'a> Decoder<'a> {
    pub fn new(callbacks: &'a mut dyn RdsDecoderCallbacks) -> Self {
        Decoder { callbacks }
    }

    pub fn decode(&mut self, _blocks: &RdsBlocks) {
        let data = RdsData::default();
        let group_type = GroupType::default();
        self.callbacks.on_oda(0, &data, &group_type);
    }
}
