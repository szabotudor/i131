pub trait Identity {
    fn ident() -> Self
    where
        Self: Sized;
}

pub fn min<T: std::cmp::PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

pub fn clamp<T: std::cmp::PartialOrd>(v: T, min: T, max: T) -> T {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}
