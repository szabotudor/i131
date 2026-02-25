use crate::*;

#[rustfmt::skip]
impl<T> Vector2Storage<T> where T: Copy {
    pub fn xx(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.x, self.x] } }
    pub fn xy(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.x, self.y] } }
    pub fn yx(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.y, self.x] } }
    pub fn yy(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.y, self.y] } }
    pub fn xxx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.x, self.x] } }
    pub fn xxy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.x, self.y] } }
    pub fn xyx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.y, self.x] } }
    pub fn xyy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.y, self.y] } }
    pub fn yxx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.x, self.x] } }
    pub fn yxy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.x, self.y] } }
    pub fn yyx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.y, self.x] } }
    pub fn yyy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.y, self.y] } }
    pub fn xxxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.x, self.x] } }
    pub fn xxxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.x, self.y] } }
    pub fn xxyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.y, self.x] } }
    pub fn xxyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.y, self.y] } }
    pub fn xyxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.x, self.x] } }
    pub fn xyxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.x, self.y] } }
    pub fn xyyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.y, self.x] } }
    pub fn xyyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.y, self.y] } }
    pub fn yxxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.x, self.x] } }
    pub fn yxxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.x, self.y] } }
    pub fn yxyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.y, self.x] } }
    pub fn yxyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.y, self.y] } }
    pub fn yyxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.x, self.x] } }
    pub fn yyxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.x, self.y] } }
    pub fn yyyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.y, self.x] } }
    pub fn yyyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.y, self.y] } }
}

#[rustfmt::skip]
impl<T> Vector3Storage<T> where T: Copy {
    pub fn xx(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.x, self.x] } }
    pub fn xy(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.x, self.y] } }
    pub fn xz(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.x, self.z] } }
    pub fn yx(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.y, self.x] } }
    pub fn yy(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.y, self.y] } }
    pub fn yz(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.y, self.z] } }
    pub fn zx(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.z, self.x] } }
    pub fn zy(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.z, self.y] } }
    pub fn zz(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.z, self.z] } }
    pub fn xxx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.x, self.x] } }
    pub fn xxy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.x, self.y] } }
    pub fn xxz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.x, self.z] } }
    pub fn xyx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.y, self.x] } }
    pub fn xyy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.y, self.y] } }
    pub fn xyz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.y, self.z] } }
    pub fn xzx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.z, self.x] } }
    pub fn xzy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.z, self.y] } }
    pub fn xzz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.z, self.z] } }
    pub fn yxx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.x, self.x] } }
    pub fn yxy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.x, self.y] } }
    pub fn yxz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.x, self.z] } }
    pub fn yyx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.y, self.x] } }
    pub fn yyy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.y, self.y] } }
    pub fn yyz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.y, self.z] } }
    pub fn yzx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.z, self.x] } }
    pub fn yzy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.z, self.y] } }
    pub fn yzz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.z, self.z] } }
    pub fn zxx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.x, self.x] } }
    pub fn zxy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.x, self.y] } }
    pub fn zxz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.x, self.z] } }
    pub fn zyx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.y, self.x] } }
    pub fn zyy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.y, self.y] } }
    pub fn zyz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.y, self.z] } }
    pub fn zzx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.z, self.x] } }
    pub fn zzy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.z, self.y] } }
    pub fn zzz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.z, self.z] } }
    pub fn xxxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.x, self.x] } }
    pub fn xxxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.x, self.y] } }
    pub fn xxxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.x, self.z] } }
    pub fn xxyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.y, self.x] } }
    pub fn xxyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.y, self.y] } }
    pub fn xxyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.y, self.z] } }
    pub fn xxzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.z, self.x] } }
    pub fn xxzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.z, self.y] } }
    pub fn xxzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.z, self.z] } }
    pub fn xyxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.x, self.x] } }
    pub fn xyxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.x, self.y] } }
    pub fn xyxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.x, self.z] } }
    pub fn xyyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.y, self.x] } }
    pub fn xyyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.y, self.y] } }
    pub fn xyyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.y, self.z] } }
    pub fn xyzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.z, self.x] } }
    pub fn xyzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.z, self.y] } }
    pub fn xyzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.z, self.z] } }
    pub fn xzxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.x, self.x] } }
    pub fn xzxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.x, self.y] } }
    pub fn xzxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.x, self.z] } }
    pub fn xzyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.y, self.x] } }
    pub fn xzyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.y, self.y] } }
    pub fn xzyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.y, self.z] } }
    pub fn xzzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.z, self.x] } }
    pub fn xzzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.z, self.y] } }
    pub fn xzzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.z, self.z] } }
    pub fn yxxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.x, self.x] } }
    pub fn yxxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.x, self.y] } }
    pub fn yxxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.x, self.z] } }
    pub fn yxyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.y, self.x] } }
    pub fn yxyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.y, self.y] } }
    pub fn yxyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.y, self.z] } }
    pub fn yxzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.z, self.x] } }
    pub fn yxzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.z, self.y] } }
    pub fn yxzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.z, self.z] } }
    pub fn yyxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.x, self.x] } }
    pub fn yyxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.x, self.y] } }
    pub fn yyxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.x, self.z] } }
    pub fn yyyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.y, self.x] } }
    pub fn yyyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.y, self.y] } }
    pub fn yyyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.y, self.z] } }
    pub fn yyzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.z, self.x] } }
    pub fn yyzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.z, self.y] } }
    pub fn yyzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.z, self.z] } }
    pub fn yzxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.x, self.x] } }
    pub fn yzxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.x, self.y] } }
    pub fn yzxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.x, self.z] } }
    pub fn yzyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.y, self.x] } }
    pub fn yzyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.y, self.y] } }
    pub fn yzyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.y, self.z] } }
    pub fn yzzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.z, self.x] } }
    pub fn yzzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.z, self.y] } }
    pub fn yzzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.z, self.z] } }
    pub fn zxxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.x, self.x] } }
    pub fn zxxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.x, self.y] } }
    pub fn zxxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.x, self.z] } }
    pub fn zxyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.y, self.x] } }
    pub fn zxyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.y, self.y] } }
    pub fn zxyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.y, self.z] } }
    pub fn zxzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.z, self.x] } }
    pub fn zxzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.z, self.y] } }
    pub fn zxzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.z, self.z] } }
    pub fn zyxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.x, self.x] } }
    pub fn zyxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.x, self.y] } }
    pub fn zyxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.x, self.z] } }
    pub fn zyyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.y, self.x] } }
    pub fn zyyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.y, self.y] } }
    pub fn zyyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.y, self.z] } }
    pub fn zyzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.z, self.x] } }
    pub fn zyzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.z, self.y] } }
    pub fn zyzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.z, self.z] } }
    pub fn zzxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.x, self.x] } }
    pub fn zzxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.x, self.y] } }
    pub fn zzxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.x, self.z] } }
    pub fn zzyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.y, self.x] } }
    pub fn zzyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.y, self.y] } }
    pub fn zzyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.y, self.z] } }
    pub fn zzzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.z, self.x] } }
    pub fn zzzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.z, self.y] } }
    pub fn zzzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.z, self.z] } }
}

#[rustfmt::skip]
impl<T> Vector4Storage<T> where T: Copy {
    pub fn xx(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.x, self.x] } }
    pub fn xy(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.x, self.y] } }
    pub fn xz(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.x, self.z] } }
    pub fn xw(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.x, self.w] } }
    pub fn yx(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.y, self.x] } }
    pub fn yy(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.y, self.y] } }
    pub fn yz(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.y, self.z] } }
    pub fn yw(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.y, self.w] } }
    pub fn zx(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.z, self.x] } }
    pub fn zy(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.z, self.y] } }
    pub fn zz(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.z, self.z] } }
    pub fn zw(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.z, self.w] } }
    pub fn wx(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.w, self.x] } }
    pub fn wy(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.w, self.y] } }
    pub fn wz(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.w, self.z] } }
    pub fn ww(&self) -> Vector<T, 2> { Vector::<T, 2> { data: [self.w, self.w] } }
    pub fn xxx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.x, self.x] } }
    pub fn xxy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.x, self.y] } }
    pub fn xxz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.x, self.z] } }
    pub fn xxw(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.x, self.w] } }
    pub fn xyx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.y, self.x] } }
    pub fn xyy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.y, self.y] } }
    pub fn xyz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.y, self.z] } }
    pub fn xyw(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.y, self.w] } }
    pub fn xzx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.z, self.x] } }
    pub fn xzy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.z, self.y] } }
    pub fn xzz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.z, self.z] } }
    pub fn xzw(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.z, self.w] } }
    pub fn xwx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.w, self.x] } }
    pub fn xwy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.w, self.y] } }
    pub fn xwz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.w, self.z] } }
    pub fn xww(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.x, self.w, self.w] } }
    pub fn yxx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.x, self.x] } }
    pub fn yxy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.x, self.y] } }
    pub fn yxz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.x, self.z] } }
    pub fn yxw(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.x, self.w] } }
    pub fn yyx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.y, self.x] } }
    pub fn yyy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.y, self.y] } }
    pub fn yyz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.y, self.z] } }
    pub fn yyw(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.y, self.w] } }
    pub fn yzx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.z, self.x] } }
    pub fn yzy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.z, self.y] } }
    pub fn yzz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.z, self.z] } }
    pub fn yzw(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.z, self.w] } }
    pub fn ywx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.w, self.x] } }
    pub fn ywy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.w, self.y] } }
    pub fn ywz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.w, self.z] } }
    pub fn yww(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.y, self.w, self.w] } }
    pub fn zxx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.x, self.x] } }
    pub fn zxy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.x, self.y] } }
    pub fn zxz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.x, self.z] } }
    pub fn zxw(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.x, self.w] } }
    pub fn zyx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.y, self.x] } }
    pub fn zyy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.y, self.y] } }
    pub fn zyz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.y, self.z] } }
    pub fn zyw(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.y, self.w] } }
    pub fn zzx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.z, self.x] } }
    pub fn zzy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.z, self.y] } }
    pub fn zzz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.z, self.z] } }
    pub fn zzw(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.z, self.w] } }
    pub fn zwx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.w, self.x] } }
    pub fn zwy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.w, self.y] } }
    pub fn zwz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.w, self.z] } }
    pub fn zww(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.z, self.w, self.w] } }
    pub fn wxx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.x, self.x] } }
    pub fn wxy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.x, self.y] } }
    pub fn wxz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.x, self.z] } }
    pub fn wxw(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.x, self.w] } }
    pub fn wyx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.y, self.x] } }
    pub fn wyy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.y, self.y] } }
    pub fn wyz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.y, self.z] } }
    pub fn wyw(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.y, self.w] } }
    pub fn wzx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.z, self.x] } }
    pub fn wzy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.z, self.y] } }
    pub fn wzz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.z, self.z] } }
    pub fn wzw(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.z, self.w] } }
    pub fn wwx(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.w, self.x] } }
    pub fn wwy(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.w, self.y] } }
    pub fn wwz(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.w, self.z] } }
    pub fn www(&self) -> Vector<T, 3> { Vector::<T, 3> { data: [self.w, self.w, self.w] } }
    pub fn xxxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.x, self.x] } }
    pub fn xxxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.x, self.y] } }
    pub fn xxxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.x, self.z] } }
    pub fn xxxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.x, self.w] } }
    pub fn xxyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.y, self.x] } }
    pub fn xxyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.y, self.y] } }
    pub fn xxyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.y, self.z] } }
    pub fn xxyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.y, self.w] } }
    pub fn xxzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.z, self.x] } }
    pub fn xxzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.z, self.y] } }
    pub fn xxzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.z, self.z] } }
    pub fn xxzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.z, self.w] } }
    pub fn xxwx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.w, self.x] } }
    pub fn xxwy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.w, self.y] } }
    pub fn xxwz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.w, self.z] } }
    pub fn xxww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.x, self.w, self.w] } }
    pub fn xyxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.x, self.x] } }
    pub fn xyxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.x, self.y] } }
    pub fn xyxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.x, self.z] } }
    pub fn xyxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.x, self.w] } }
    pub fn xyyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.y, self.x] } }
    pub fn xyyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.y, self.y] } }
    pub fn xyyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.y, self.z] } }
    pub fn xyyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.y, self.w] } }
    pub fn xyzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.z, self.x] } }
    pub fn xyzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.z, self.y] } }
    pub fn xyzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.z, self.z] } }
    pub fn xyzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.z, self.w] } }
    pub fn xywx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.w, self.x] } }
    pub fn xywy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.w, self.y] } }
    pub fn xywz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.w, self.z] } }
    pub fn xyww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.y, self.w, self.w] } }
    pub fn xzxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.x, self.x] } }
    pub fn xzxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.x, self.y] } }
    pub fn xzxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.x, self.z] } }
    pub fn xzxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.x, self.w] } }
    pub fn xzyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.y, self.x] } }
    pub fn xzyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.y, self.y] } }
    pub fn xzyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.y, self.z] } }
    pub fn xzyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.y, self.w] } }
    pub fn xzzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.z, self.x] } }
    pub fn xzzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.z, self.y] } }
    pub fn xzzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.z, self.z] } }
    pub fn xzzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.z, self.w] } }
    pub fn xzwx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.w, self.x] } }
    pub fn xzwy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.w, self.y] } }
    pub fn xzwz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.w, self.z] } }
    pub fn xzww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.z, self.w, self.w] } }
    pub fn xwxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.x, self.x] } }
    pub fn xwxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.x, self.y] } }
    pub fn xwxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.x, self.z] } }
    pub fn xwxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.x, self.w] } }
    pub fn xwyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.y, self.x] } }
    pub fn xwyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.y, self.y] } }
    pub fn xwyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.y, self.z] } }
    pub fn xwyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.y, self.w] } }
    pub fn xwzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.z, self.x] } }
    pub fn xwzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.z, self.y] } }
    pub fn xwzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.z, self.z] } }
    pub fn xwzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.z, self.w] } }
    pub fn xwwx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.w, self.x] } }
    pub fn xwwy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.w, self.y] } }
    pub fn xwwz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.w, self.z] } }
    pub fn xwww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.x, self.w, self.w, self.w] } }
    pub fn yxxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.x, self.x] } }
    pub fn yxxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.x, self.y] } }
    pub fn yxxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.x, self.z] } }
    pub fn yxxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.x, self.w] } }
    pub fn yxyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.y, self.x] } }
    pub fn yxyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.y, self.y] } }
    pub fn yxyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.y, self.z] } }
    pub fn yxyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.y, self.w] } }
    pub fn yxzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.z, self.x] } }
    pub fn yxzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.z, self.y] } }
    pub fn yxzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.z, self.z] } }
    pub fn yxzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.z, self.w] } }
    pub fn yxwx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.w, self.x] } }
    pub fn yxwy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.w, self.y] } }
    pub fn yxwz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.w, self.z] } }
    pub fn yxww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.x, self.w, self.w] } }
    pub fn yyxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.x, self.x] } }
    pub fn yyxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.x, self.y] } }
    pub fn yyxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.x, self.z] } }
    pub fn yyxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.x, self.w] } }
    pub fn yyyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.y, self.x] } }
    pub fn yyyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.y, self.y] } }
    pub fn yyyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.y, self.z] } }
    pub fn yyyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.y, self.w] } }
    pub fn yyzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.z, self.x] } }
    pub fn yyzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.z, self.y] } }
    pub fn yyzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.z, self.z] } }
    pub fn yyzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.z, self.w] } }
    pub fn yywx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.w, self.x] } }
    pub fn yywy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.w, self.y] } }
    pub fn yywz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.w, self.z] } }
    pub fn yyww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.y, self.w, self.w] } }
    pub fn yzxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.x, self.x] } }
    pub fn yzxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.x, self.y] } }
    pub fn yzxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.x, self.z] } }
    pub fn yzxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.x, self.w] } }
    pub fn yzyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.y, self.x] } }
    pub fn yzyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.y, self.y] } }
    pub fn yzyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.y, self.z] } }
    pub fn yzyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.y, self.w] } }
    pub fn yzzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.z, self.x] } }
    pub fn yzzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.z, self.y] } }
    pub fn yzzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.z, self.z] } }
    pub fn yzzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.z, self.w] } }
    pub fn yzwx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.w, self.x] } }
    pub fn yzwy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.w, self.y] } }
    pub fn yzwz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.w, self.z] } }
    pub fn yzww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.z, self.w, self.w] } }
    pub fn ywxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.x, self.x] } }
    pub fn ywxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.x, self.y] } }
    pub fn ywxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.x, self.z] } }
    pub fn ywxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.x, self.w] } }
    pub fn ywyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.y, self.x] } }
    pub fn ywyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.y, self.y] } }
    pub fn ywyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.y, self.z] } }
    pub fn ywyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.y, self.w] } }
    pub fn ywzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.z, self.x] } }
    pub fn ywzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.z, self.y] } }
    pub fn ywzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.z, self.z] } }
    pub fn ywzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.z, self.w] } }
    pub fn ywwx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.w, self.x] } }
    pub fn ywwy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.w, self.y] } }
    pub fn ywwz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.w, self.z] } }
    pub fn ywww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.y, self.w, self.w, self.w] } }
    pub fn zxxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.x, self.x] } }
    pub fn zxxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.x, self.y] } }
    pub fn zxxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.x, self.z] } }
    pub fn zxxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.x, self.w] } }
    pub fn zxyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.y, self.x] } }
    pub fn zxyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.y, self.y] } }
    pub fn zxyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.y, self.z] } }
    pub fn zxyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.y, self.w] } }
    pub fn zxzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.z, self.x] } }
    pub fn zxzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.z, self.y] } }
    pub fn zxzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.z, self.z] } }
    pub fn zxzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.z, self.w] } }
    pub fn zxwx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.w, self.x] } }
    pub fn zxwy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.w, self.y] } }
    pub fn zxwz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.w, self.z] } }
    pub fn zxww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.x, self.w, self.w] } }
    pub fn zyxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.x, self.x] } }
    pub fn zyxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.x, self.y] } }
    pub fn zyxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.x, self.z] } }
    pub fn zyxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.x, self.w] } }
    pub fn zyyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.y, self.x] } }
    pub fn zyyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.y, self.y] } }
    pub fn zyyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.y, self.z] } }
    pub fn zyyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.y, self.w] } }
    pub fn zyzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.z, self.x] } }
    pub fn zyzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.z, self.y] } }
    pub fn zyzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.z, self.z] } }
    pub fn zyzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.z, self.w] } }
    pub fn zywx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.w, self.x] } }
    pub fn zywy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.w, self.y] } }
    pub fn zywz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.w, self.z] } }
    pub fn zyww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.y, self.w, self.w] } }
    pub fn zzxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.x, self.x] } }
    pub fn zzxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.x, self.y] } }
    pub fn zzxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.x, self.z] } }
    pub fn zzxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.x, self.w] } }
    pub fn zzyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.y, self.x] } }
    pub fn zzyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.y, self.y] } }
    pub fn zzyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.y, self.z] } }
    pub fn zzyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.y, self.w] } }
    pub fn zzzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.z, self.x] } }
    pub fn zzzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.z, self.y] } }
    pub fn zzzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.z, self.z] } }
    pub fn zzzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.z, self.w] } }
    pub fn zzwx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.w, self.x] } }
    pub fn zzwy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.w, self.y] } }
    pub fn zzwz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.w, self.z] } }
    pub fn zzww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.z, self.w, self.w] } }
    pub fn zwxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.x, self.x] } }
    pub fn zwxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.x, self.y] } }
    pub fn zwxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.x, self.z] } }
    pub fn zwxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.x, self.w] } }
    pub fn zwyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.y, self.x] } }
    pub fn zwyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.y, self.y] } }
    pub fn zwyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.y, self.z] } }
    pub fn zwyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.y, self.w] } }
    pub fn zwzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.z, self.x] } }
    pub fn zwzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.z, self.y] } }
    pub fn zwzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.z, self.z] } }
    pub fn zwzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.z, self.w] } }
    pub fn zwwx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.w, self.x] } }
    pub fn zwwy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.w, self.y] } }
    pub fn zwwz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.w, self.z] } }
    pub fn zwww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.z, self.w, self.w, self.w] } }
    pub fn wxxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.x, self.x] } }
    pub fn wxxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.x, self.y] } }
    pub fn wxxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.x, self.z] } }
    pub fn wxxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.x, self.w] } }
    pub fn wxyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.y, self.x] } }
    pub fn wxyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.y, self.y] } }
    pub fn wxyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.y, self.z] } }
    pub fn wxyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.y, self.w] } }
    pub fn wxzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.z, self.x] } }
    pub fn wxzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.z, self.y] } }
    pub fn wxzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.z, self.z] } }
    pub fn wxzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.z, self.w] } }
    pub fn wxwx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.w, self.x] } }
    pub fn wxwy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.w, self.y] } }
    pub fn wxwz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.w, self.z] } }
    pub fn wxww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.x, self.w, self.w] } }
    pub fn wyxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.x, self.x] } }
    pub fn wyxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.x, self.y] } }
    pub fn wyxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.x, self.z] } }
    pub fn wyxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.x, self.w] } }
    pub fn wyyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.y, self.x] } }
    pub fn wyyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.y, self.y] } }
    pub fn wyyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.y, self.z] } }
    pub fn wyyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.y, self.w] } }
    pub fn wyzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.z, self.x] } }
    pub fn wyzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.z, self.y] } }
    pub fn wyzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.z, self.z] } }
    pub fn wyzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.z, self.w] } }
    pub fn wywx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.w, self.x] } }
    pub fn wywy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.w, self.y] } }
    pub fn wywz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.w, self.z] } }
    pub fn wyww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.y, self.w, self.w] } }
    pub fn wzxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.x, self.x] } }
    pub fn wzxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.x, self.y] } }
    pub fn wzxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.x, self.z] } }
    pub fn wzxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.x, self.w] } }
    pub fn wzyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.y, self.x] } }
    pub fn wzyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.y, self.y] } }
    pub fn wzyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.y, self.z] } }
    pub fn wzyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.y, self.w] } }
    pub fn wzzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.z, self.x] } }
    pub fn wzzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.z, self.y] } }
    pub fn wzzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.z, self.z] } }
    pub fn wzzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.z, self.w] } }
    pub fn wzwx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.w, self.x] } }
    pub fn wzwy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.w, self.y] } }
    pub fn wzwz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.w, self.z] } }
    pub fn wzww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.z, self.w, self.w] } }
    pub fn wwxx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.x, self.x] } }
    pub fn wwxy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.x, self.y] } }
    pub fn wwxz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.x, self.z] } }
    pub fn wwxw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.x, self.w] } }
    pub fn wwyx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.y, self.x] } }
    pub fn wwyy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.y, self.y] } }
    pub fn wwyz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.y, self.z] } }
    pub fn wwyw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.y, self.w] } }
    pub fn wwzx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.z, self.x] } }
    pub fn wwzy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.z, self.y] } }
    pub fn wwzz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.z, self.z] } }
    pub fn wwzw(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.z, self.w] } }
    pub fn wwwx(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.w, self.x] } }
    pub fn wwwy(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.w, self.y] } }
    pub fn wwwz(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.w, self.z] } }
    pub fn wwww(&self) -> Vector<T, 4> { Vector::<T, 4> { data: [self.w, self.w, self.w, self.w] } }
}
