use std::hash::Hash;

pub struct PtrHash<'a, T>(&'a T);

impl<T> Clone for PtrHash<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for PtrHash<'_, T> {}

impl<'a, T> From<&'a T> for PtrHash<'a, T> {
    fn from(value: &'a T) -> Self {
        PtrHash(value)
    }
}

impl<T> Hash for PtrHash<'_, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0, state);
    }
}

impl<T> PartialEq for PtrHash<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<T> Eq for PtrHash<'_, T> {}

impl<T> std::ops::Deref for PtrHash<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
