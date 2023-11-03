use std::{marker::PhantomData, ptr::NonNull};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeHint {
    Float,
    Hex,
    Bin,
    Int,
}

#[derive(Clone, Copy)]
pub struct Number<'a> {
    ptr: NonNull<u8>,
    len: u16,
    ext_back_off: u8,
    hint: TypeHint,
    _phan: PhantomData<&'a str>,
}

impl PartialEq for Number<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.get_full() == other.get_full()
    }
}

impl Eq for Number<'_> {
    fn assert_receiver_is_total_eq(&self) {}
}

impl std::fmt::Debug for Number<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Number")
            .field("num", &self.get_num())
            .field("suffix", &self.get_suffix())
            .field("hint", &self.hint)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberError {
    LenTooLong,
    SuffixTooLong,
    InvalidNumLen,
}

impl<'a> Number<'a> {
    pub fn new(str: &'a str, hint: TypeHint) -> Result<Self, NumberError> {
        Ok(Self {
            ptr: unsafe { NonNull::new_unchecked(str.as_ptr().cast_mut()) },
            len: str.len().try_into().map_err(|_| NumberError::LenTooLong)?,
            ext_back_off: 0,
            hint,
            _phan: PhantomData,
        })
    }

    pub fn new_with_suffix(
        str: &'a str,
        num_len: usize,
        hint: TypeHint,
    ) -> Result<Self, NumberError> {
        match num_len.cmp(&str.len()) {
            std::cmp::Ordering::Greater => Err(NumberError::InvalidNumLen),
            std::cmp::Ordering::Equal => Self::new(str, hint),
            std::cmp::Ordering::Less => Ok(Self {
                ptr: unsafe { NonNull::new_unchecked(str.as_ptr().cast_mut()) },
                len: str.len().try_into().map_err(|_| NumberError::LenTooLong)?,
                ext_back_off: (str.len() - num_len)
                    .try_into()
                    .map_err(|_| NumberError::SuffixTooLong)?,
                hint,
                _phan: PhantomData,
            }),
        }
    }

    pub fn get_hint(&self) -> TypeHint {
        self.hint
    }

    pub fn get_num(&self) -> &'a str {
        unsafe {
            let buf = std::slice::from_raw_parts(
                self.ptr.as_ptr(),
                self.len as usize - self.ext_back_off as usize,
            );
            std::str::from_utf8_unchecked(buf)
        }
    }

    pub fn get_suffix(&self) -> Option<&'a str> {
        if self.ext_back_off == 0 {
            None
        } else {
            unsafe {
                let start = self
                    .ptr
                    .as_ptr()
                    .add(self.len as usize)
                    .sub(self.ext_back_off as usize);
                let buf = std::slice::from_raw_parts(start, self.ext_back_off as usize);
                Some(std::str::from_utf8_unchecked(buf))
            }
        }
    }

    pub fn get_full(&self) -> &'a str {
        unsafe {
            let buf = std::slice::from_raw_parts(self.ptr.as_ptr(), self.len as usize);
            std::str::from_utf8_unchecked(buf)
        }
    }
}
