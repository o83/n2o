use std::fmt;
use std::ops;

#[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Ready(pub usize);

impl Ready {
    pub fn none() -> Ready {
        Ready(0)
    }

    #[inline]
    pub fn readable() -> Ready {
        Ready(0x001)
    }

    #[inline]
    pub fn writable() -> Ready {
        Ready(0x002)
    }

    #[inline]
    pub fn error() -> Ready {
        Ready(0x004)
    }

    #[inline]
    pub fn hup() -> Ready {
        Ready(0x008)
    }

    // Private
    #[inline]
    pub fn drop() -> Ready {
        Ready(0x10)
    }

    #[inline]
    pub fn all() -> Ready {
        Ready::readable() | Ready::writable() | Ready::hup() | Ready::error()
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        (*self & !Ready::drop()) == Ready::none()
    }

    #[inline]
    pub fn is_readable(&self) -> bool {
        self.contains(Ready::readable())
    }

    #[inline]
    pub fn is_writable(&self) -> bool {
        self.contains(Ready::writable())
    }

    #[inline]
    pub fn is_error(&self) -> bool {
        self.contains(Ready::error())
    }

    #[inline]
    pub fn is_hup(&self) -> bool {
        self.contains(Ready::hup())
    }

    #[inline]
    pub fn insert(&mut self, other: Ready) {
        self.0 |= other.0;
    }

    #[inline]
    pub fn remove(&mut self, other: Ready) {
        self.0 &= !other.0;
    }

    #[inline]
    pub fn bits(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn contains(&self, other: Ready) -> bool {
        (*self & other) == other
    }
}

impl fmt::Debug for Ready {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut one = false;
        let flags = [(Ready::readable(), "Readable"),
                     (Ready::writable(), "Writable"),
                     (Ready::error(), "Error"),
                     (Ready::hup(), "Hup"),
                     (Ready::drop(), "Drop")];

        try!(write!(fmt, "Ready {{"));

        for &(flag, msg) in &flags {
            if self.contains(flag) {
                if one {
                    try!(write!(fmt, " | "))
                }
                try!(write!(fmt, "{}", msg));

                one = true
            }
        }

        try!(write!(fmt, "}}"));

        Ok(())
    }
}

impl ops::BitOr for Ready {
    type Output = Ready;

    #[inline]
    fn bitor(self, other: Ready) -> Ready {
        Ready(self.bits() | other.bits())
    }
}

impl ops::BitAnd for Ready {
    type Output = Ready;

    #[inline]
    fn bitand(self, other: Ready) -> Ready {
        Ready(self.bits() & other.bits())
    }
}

impl ops::Not for Ready {
    type Output = Ready;

    #[inline]
    fn not(self) -> Ready {
        Ready(!self.bits() & Ready::all().bits())
    }
}

