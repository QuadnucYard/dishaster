use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct Modified<T> {
    value: T,
    modified: bool,
}

impl<T> Modified<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            modified: true,
        }
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        self.modified = true;
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn reset_modified(&mut self) {
        self.modified = false;
    }

    pub fn take_new_value(&mut self) -> Option<&T> {
        if self.modified {
            self.modified = false;
            Some(&self.value)
        } else {
            None
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Modified<U> {
        Modified {
            value: f(self.value),
            modified: self.modified,
        }
    }
}

impl<T: Default> Default for Modified<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> Deref for Modified<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Modified<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.modified = true;
        &mut self.value
    }
}

impl<T> From<T> for Modified<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
