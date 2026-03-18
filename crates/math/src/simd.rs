use crate::Vec4f32;
use std::{
    arch::x86_64::{__m128, _mm_add_ps, _mm_div_ps, _mm_mul_ps, _mm_sub_ps},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

//=============
// f32 Vectors
//=============

impl Add for Vec4f32 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe {
            let lhs = std::mem::transmute::<[f32; 4], __m128>(self.data);
            let rhs = std::mem::transmute::<[f32; 4], __m128>(rhs.data);
            let res = _mm_add_ps(lhs, rhs);
            Self {
                data: std::mem::transmute::<__m128, [f32; 4]>(res),
            }
        }
    }
}
impl Sub for Vec4f32 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        unsafe {
            let lhs = std::mem::transmute::<[f32; 4], __m128>(self.data);
            let rhs = std::mem::transmute::<[f32; 4], __m128>(rhs.data);
            let res = _mm_sub_ps(lhs, rhs);
            Self {
                data: std::mem::transmute::<__m128, [f32; 4]>(res),
            }
        }
    }
}
impl Mul for Vec4f32 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        unsafe {
            let lhs = std::mem::transmute::<[f32; 4], __m128>(self.data);
            let rhs = std::mem::transmute::<[f32; 4], __m128>(rhs.data);
            let res = _mm_mul_ps(lhs, rhs);
            Self {
                data: std::mem::transmute::<__m128, [f32; 4]>(res),
            }
        }
    }
}
impl Div for Vec4f32 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        unsafe {
            let lhs = std::mem::transmute::<[f32; 4], __m128>(self.data);
            let rhs = std::mem::transmute::<[f32; 4], __m128>(rhs.data);
            let res = _mm_div_ps(lhs, rhs);
            Self {
                data: std::mem::transmute::<__m128, [f32; 4]>(res),
            }
        }
    }
}

// x86_64 has no in-place operators, so we can just use the normal operators
impl AddAssign for Vec4f32 {
    fn add_assign(&mut self, rhs: Self) {
        unsafe {
            let lhs = std::mem::transmute::<[f32; 4], __m128>(self.data);
            let rhs = std::mem::transmute::<[f32; 4], __m128>(rhs.data);
            let res = _mm_add_ps(lhs, rhs);
            self.data = std::mem::transmute::<__m128, [f32; 4]>(res);
        }
    }
}
impl SubAssign for Vec4f32 {
    fn sub_assign(&mut self, rhs: Self) {
        unsafe {
            let lhs = std::mem::transmute::<[f32; 4], __m128>(self.data);
            let rhs = std::mem::transmute::<[f32; 4], __m128>(rhs.data);
            let res = _mm_sub_ps(lhs, rhs);
            self.data = std::mem::transmute::<__m128, [f32; 4]>(res);
        }
    }
}
impl MulAssign for Vec4f32 {
    fn mul_assign(&mut self, rhs: Self) {
        unsafe {
            let lhs = std::mem::transmute::<[f32; 4], __m128>(self.data);
            let rhs = std::mem::transmute::<[f32; 4], __m128>(rhs.data);
            let res = _mm_mul_ps(lhs, rhs);
            self.data = std::mem::transmute::<__m128, [f32; 4]>(res);
        }
    }
}
impl DivAssign for Vec4f32 {
    fn div_assign(&mut self, rhs: Self) {
        unsafe {
            let lhs = std::mem::transmute::<[f32; 4], __m128>(self.data);
            let rhs = std::mem::transmute::<[f32; 4], __m128>(rhs.data);
            let res = _mm_div_ps(lhs, rhs);
            self.data = std::mem::transmute::<__m128, [f32; 4]>(res);
        }
    }
}
