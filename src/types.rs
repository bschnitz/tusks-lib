use std::ops::{Deref, DerefMut};

// ============================================================================
// Repeat<T, N> - Vec mit Typ-Level Zahl (keine Runtime-Prüfung)
// ============================================================================

pub struct Repeat<T, const N: usize>(Vec<T>);

impl<T, const N: usize> Repeat<T, N> {
    pub fn new(vec: Vec<T>) -> Self {
        Repeat(vec)
    }
    
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T, const N: usize> Deref for Repeat<T, N> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> DerefMut for Repeat<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, const N: usize> From<Vec<T>> for Repeat<T, N> {
    fn from(vec: Vec<T>) -> Self {
        Repeat(vec)
    }
}

// ============================================================================
// RepeatMin<T, MIN> - Vec mit Mindestlänge (keine Runtime-Prüfung)
// ============================================================================

pub struct RepeatMin<T, const MIN: usize>(Vec<T>);

impl<T, const MIN: usize> RepeatMin<T, MIN> {
    pub fn new(vec: Vec<T>) -> Self {
        RepeatMin(vec)
    }
    
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T, const MIN: usize> Deref for RepeatMin<T, MIN> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const MIN: usize> DerefMut for RepeatMin<T, MIN> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, const MIN: usize> From<Vec<T>> for RepeatMin<T, MIN> {
    fn from(vec: Vec<T>) -> Self {
        RepeatMin(vec)
    }
}

// ============================================================================
// RepeatMax<T, MAX> - Vec mit Maximallänge (keine Runtime-Prüfung)
// ============================================================================

pub struct RepeatMax<T, const MAX: usize>(Vec<T>);

impl<T, const MAX: usize> RepeatMax<T, MAX> {
    pub fn new(vec: Vec<T>) -> Self {
        RepeatMax(vec)
    }
    
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T, const MAX: usize> Deref for RepeatMax<T, MAX> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const MAX: usize> DerefMut for RepeatMax<T, MAX> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, const MAX: usize> From<Vec<T>> for RepeatMax<T, MAX> {
    fn from(vec: Vec<T>) -> Self {
        RepeatMax(vec)
    }
}

// ============================================================================
// RepeatMinMax<T, MIN, MAX> - Vec mit Min/Max Länge (keine Runtime-Prüfung)
// ============================================================================

pub struct RepeatMinMax<T, const MIN: usize, const MAX: usize>(Vec<T>);

impl<T, const MIN: usize, const MAX: usize> RepeatMinMax<T, MIN, MAX> {
    pub fn new(vec: Vec<T>) -> Self {
        RepeatMinMax(vec)
    }
    
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T, const MIN: usize, const MAX: usize> Deref for RepeatMinMax<T, MIN, MAX> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const MIN: usize, const MAX: usize> DerefMut for RepeatMinMax<T, MIN, MAX> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, const MIN: usize, const MAX: usize> From<Vec<T>> for RepeatMinMax<T, MIN, MAX> {
    fn from(vec: Vec<T>) -> Self {
        RepeatMinMax(vec)
    }
}
