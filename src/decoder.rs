use crate::callbacks::{RdsData, RdsDecoderCallbacks, RdsGroupType};

pub enum BlockErrorCount {
    None = 0,        // No block errors.
    OneToTwo = 1,    // 1-2 block errors.
    ThreeToFive = 2, // 3-5 block errors.
    SixPlus = 3,     // 6+ block errors.
}

pub struct RdsBlock {
    pub block: u16,                  // The RDS block data.
    pub num_errors: BlockErrorCount, // Number of bit errors in the block.
}

pub struct RdsBlocks {
    pub a: Option<RdsBlock>,
    pub b: Option<RdsBlock>,
    pub c: Option<RdsBlock>,
    pub d: Option<RdsBlock>,
}

pub struct Decoder<'a> {
    callbacks: &'a mut dyn RdsDecoderCallbacks,
}

impl<'a> Decoder<'a> {
    pub fn new(callbacks: &'a mut dyn RdsDecoderCallbacks) -> Self {
        Decoder { callbacks }
    }

    pub fn decode(&mut self, blocks: &RdsBlocks) {
        let data = RdsData::default();
        let group_type = RdsGroupType::default();
        self.callbacks.on_oda(0, &data, &group_type);
    }
}
