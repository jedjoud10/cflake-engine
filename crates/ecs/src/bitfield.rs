use bitfield::BitfieldU32;

// A component bitfield 
#[derive(Default, Clone)]
pub struct ComponentBitfield {
    pub bitfield: BitfieldU32
}

// A system bitfield
#[derive(Default, Clone)]
pub struct SystemBitfield {
    pub bitfield: BitfieldU32
}

