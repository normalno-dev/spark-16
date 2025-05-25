pub struct Register(u8);

impl Register {
    pub fn idx(self) -> u8 {
        self.0
    }

    pub fn new(id: u8) -> Result<Self, &'static str> {
        if id > 0x7 {
            Err("register must be 0-7")
        } else {
            Ok(Self(id))
        }
    }
}

impl Into<u8> for Register {
    fn into(self) -> u8 {
        todo!()
    }
}

impl TryFrom<u8> for Register {  
    type Error = &'static str;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<u16> for Register {  
    type Error = &'static str;
    
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value as u8)
    }
}