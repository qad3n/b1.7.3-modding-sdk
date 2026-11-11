pub use crate::terrain::math::{cos, floor, sin, sqrt};
use mod_protocol::abi::BlockPos;

pub type Vec3 = [f64; 3];

// the float pi EntityLiving.faceEntity divides by, so angles match vanilla to the last bit
const FLOAT_PI: f64 = std::f32::consts::PI as f64;
const HALF_TURN_DEGREES: f64 = 180.0;
const QUARTER_TURN_DEGREES: f32 = 90.0;

pub fn add(a: Vec3, b: Vec3) -> Vec3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }

pub fn sub(a: Vec3, b: Vec3) -> Vec3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }

pub fn scale(a: Vec3, by: f64) -> Vec3 { [a[0] * by, a[1] * by, a[2] * by] }

pub fn dot(a: Vec3, b: Vec3) -> f64 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }

pub fn lengthSquared(a: Vec3) -> f64 { dot(a, a) }

// Vec3D.lengthVector, through the float sqrt_double
pub fn length(a: Vec3) -> f64 { f64::from(sqrt(lengthSquared(a))) }

pub fn distanceSquared(a: Vec3, b: Vec3) -> f64 { lengthSquared(sub(a, b)) }

// Vec3D.distanceTo and Entity.getDistance, through the float sqrt_double
pub fn distance(a: Vec3, b: Vec3) -> f64 { length(sub(a, b)) }

// Vec3D.normalize divides each axis by the float length and zeroes a vector shorter than 1.0E-4
pub fn normalize(a: Vec3) -> Vec3
{
    const SHORTEST: f64 = 1.0e-4;

    let length: f64 = length(a);
    if length < SHORTEST
    {
        return [0.0; 3];
    }

    a.map(|axis| axis / length)
}

pub fn lerp(from: Vec3, to: Vec3, t: f64) -> Vec3 { add(from, scale(sub(to, from), t)) }

// the block a point lies in, MathHelper.floor_double on each axis
pub fn blockOf(pos: Vec3) -> BlockPos { BlockPos::new(floor(pos[0]), floor(pos[1]), floor(pos[2])) }

pub fn centerOf(pos: BlockPos) -> Vec3
{
    const HALF: f64 = 0.5;

    [pos.x as f64 + HALF, pos.y as f64 + HALF, pos.z as f64 + HALF]
}

pub fn offset(pos: BlockPos, face: Face) -> BlockPos
{
    let [dx, dy, dz] = face.normal();
    pos.offset(dx, dy, dz)
}

// the yaw EntityLiving.faceEntity turns to so it looks from one point toward another
pub fn yawTowards(from: Vec3, to: Vec3) -> f32
{
    let dx: f64 = to[0] - from[0];
    let dz: f64 = to[2] - from[2];
    (dz.atan2(dx) * HALF_TURN_DEGREES / FLOAT_PI) as f32 - QUARTER_TURN_DEGREES
}

// the pitch EntityLiving.faceEntity tilts to, negative looks up
pub fn pitchTowards(from: Vec3, to: Vec3) -> f32
{
    let dx: f64 = to[0] - from[0];
    let dy: f64 = to[1] - from[1];
    let dz: f64 = to[2] - from[2];
    let flat: f64 = (dx * dx + dz * dz).sqrt();
    -((dy.atan2(flat) * HALF_TURN_DEGREES / FLOAT_PI) as f32)
}

// the six block faces in net.minecraft.src.Facing order, the index every face parameter of the API uses
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Face
{
    Down,
    Up,
    North,
    South,
    West,
    East,
}

struct FaceRow
{
    m_face: Face,
    m_normal: [i32; 3],
    m_opposite: Face,
}

const FACES: [FaceRow; 6] = [
    FaceRow { m_face: Face::Down, m_normal: [0, -1, 0], m_opposite: Face::Up },
    FaceRow { m_face: Face::Up, m_normal: [0, 1, 0], m_opposite: Face::Down },
    FaceRow { m_face: Face::North, m_normal: [0, 0, -1], m_opposite: Face::South },
    FaceRow { m_face: Face::South, m_normal: [0, 0, 1], m_opposite: Face::North },
    FaceRow { m_face: Face::West, m_normal: [-1, 0, 0], m_opposite: Face::East },
    FaceRow { m_face: Face::East, m_normal: [1, 0, 0], m_opposite: Face::West },
];

impl Face
{
    pub const ALL: [Face; 6] = [Face::Down, Face::Up, Face::North, Face::South, Face::West, Face::East];

    pub fn fromIndex(index: i32) -> Option<Face>
    {
        usize::try_from(index).ok().and_then(|index| FACES.get(index)).map(|row| row.m_face)
    }

    pub fn index(self) -> i32 { self as i32 }

    pub fn normal(self) -> [i32; 3] { self.row().m_normal }

    pub fn opposite(self) -> Face { self.row().m_opposite }

    fn row(self) -> &'static FaceRow { &FACES[self as usize] }
}

#[cfg(test)]
mod tests
{
    use super::*;

    // input, Vec3D.lengthVector bits and Vec3D.normalize bits from Oracle Java 6u26 running the b1.7.3 bodies
    const VANILLA: [(Vec3, u64, [u64; 3]); 6] = [
        ([0.3, -1.7, 2.9], 0x400AFFCF80000000, [0x3FB6C194F73A0BBC, 0xBFE01E742F1E72FB, 0x3FEB7F3EAABB78D9]),
        ([12.25, 0.1, -7.5], 0x402CBA5520000000, [0x3FEB4A63737DE25E, 0x3F7C84102515D269, 0xBFE0B56175BAC949]),
        ([-0.6, 0.8, 0.05], 0x3FF0051DE0000000, [0xBFE32D114FA3FD40, 0x3FE9916C6A2FFC56, 0x3FA9916C6A2FFC56]),
        ([1.0e-5, 2.0e-5, 0.0], 0x3EF7726640000000, [0x0, 0x0, 0x0]),
        ([3.0, 4.0, 12.0], 0x402A000000000000, [0x3FCD89D89D89D89E, 0x3FD3B13B13B13B14, 0x3FED89D89D89D89E]),
        ([-120.37, 64.5, 33.125], 0x406190B4C0000000, [0xBFEB69327DCB9A23, 0x3FDD604CA7BC9DB3, 0x3FCE2C56AC49ABE6]),
    ];

    #[test]
    fn length_matches_vec3d_length_vector()
    {
        for (vector, bits, _) in VANILLA
        {
            assert_eq!(length(vector).to_bits(), bits, "length of {vector:?}");
            assert_eq!(distance(vector, [0.0; 3]).to_bits(), bits, "distance of {vector:?}");
        }
    }

    #[test]
    fn normalize_matches_vec3d_normalize()
    {
        for (vector, _, bits) in VANILLA
        {
            assert_eq!(normalize(vector).map(f64::to_bits), bits, "normalize of {vector:?}");
        }
    }
}
