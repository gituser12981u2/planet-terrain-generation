use crate::terrain_generation::terrain_core::Face;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeKey {
    pub face: Face,
    pub level: u8,
    pub ix: u32,
    pub iy: u32,
}

impl NodeKey {
    #[inline]
    pub fn root(face: Face) -> Self {
        Self {
            face,
            level: 0,
            ix: 0,
            iy: 0,
        }
    }

    #[inline]
    pub fn uv_rect(self) -> (f32, f32, f32, f32) {
        // u,v in [-1, 1], subdivided into 2^level
        let div = 1u32 << self.level;
        let du = 2.0 / div as f32;
        let dv = 2.0 / div as f32;
        let u0 = -1.0 + self.ix as f32 * du;
        let v0 = -1.0 + self.iy as f32 * dv;
        (u0, v0, du, dv)
    }
}
