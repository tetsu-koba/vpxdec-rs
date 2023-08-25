use std::error::Error;

pub struct VpxDec {
    // stub
    rawvideo: Vec<u8>,
}

impl VpxDec {
    pub fn init(_fourcc: &[u8; 4]) -> Result<Self, Box<dyn Error>> {
        // stub
        Ok(VpxDec {
            rawvideo: vec![0u8; 10],
        })
    }

    pub fn decode(&mut self, _frame_buffer: &[u8]) -> Result<&[u8], Box<dyn Error>> {
        // stub
        Ok(&self.rawvideo)
    }

    // ... other necessary methods
}
