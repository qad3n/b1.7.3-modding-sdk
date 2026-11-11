mod octaves;
mod perlin;
pub mod structure;

pub use octaves::NoiseGeneratorOctaves;
pub use perlin::NoiseGeneratorPerlin;
use std::cell::Cell;

mod lcg
{
    pub const MULTIPLIER: i64 = 0x5DEECE66D;
    pub const ADDEND: i64 = 0xB;
    pub const MASK: i64 = (1 << 48) - 1;
}

/// java.util.Random, so ported generators draw the same numbers as the Java game
pub struct Random
{
    m_seed: Cell<i64>,
    m_gaussian: Cell<Option<f64>>,
}

impl Random
{
    pub fn withSeed(seed: i64) -> Self
    {
        let random: Self = Self { m_seed: Cell::new(0), m_gaussian: Cell::new(None) };
        random.setSeed(seed);
        random
    }

    pub fn setSeed(&self, seed: i64)
    {
        self.m_seed.set((seed ^ lcg::MULTIPLIER) & lcg::MASK);
        self.m_gaussian.set(None);
    }

    pub fn next(&self, bits: i32) -> i32
    {
        let next: i64 = (self.m_seed.get().wrapping_mul(lcg::MULTIPLIER).wrapping_add(lcg::ADDEND)) & lcg::MASK;
        self.m_seed.set(next);
        ((next as u64) >> (48 - bits)) as i32
    }

    pub fn nextInt(&self) -> i32 { self.next(32) }

    /// java throws for a bound below one, ported code never passes one so it returns zero instead
    pub fn nextIntBounded(&self, n: i32) -> i32
    {
        if n <= 0
        {
            return 0;
        }
        if (n & n.wrapping_neg()) == n
        {
            return ((n as i64).wrapping_mul(self.next(31) as i64) >> 31) as i32;
        }

        loop
        {
            let bits: i32 = self.next(31);
            let value: i32 = bits % n;
            if bits.wrapping_sub(value).wrapping_add(n - 1) >= 0
            {
                return value;
            }
        }
    }

    pub fn nextLong(&self) -> i64 { ((self.next(32) as i64) << 32).wrapping_add(self.next(32) as i64) }

    pub fn nextBoolean(&self) -> bool { self.next(1) != 0 }

    pub fn nextFloat(&self) -> f32 { self.next(24) as f32 / (1 << 24) as f32 }

    pub fn nextDouble(&self) -> f64
    {
        (((self.next(26) as i64) << 27) + self.next(27) as i64) as f64 / (1i64 << 53) as f64
    }

    pub fn nextGaussian(&self) -> f64
    {
        if let Some(next) = self.m_gaussian.take()
        {
            return next;
        }

        loop
        {
            let v1: f64 = 2.0 * self.nextDouble() - 1.0;
            let v2: f64 = 2.0 * self.nextDouble() - 1.0;
            let s: f64 = v1 * v1 + v2 * v2;
            if s >= 1.0 || s == 0.0
            {
                continue;
            }

            let multiplier: f64 = (-2.0 * s.ln() / s).sqrt();
            self.m_gaussian.set(Some(v2 * multiplier));
            return v1 * multiplier;
        }
    }
}

pub mod chunk
{
    pub const WIDTH: i32 = 16;
    pub const HEIGHT: i32 = 128;
    pub const BLOCK_COUNT: usize = 32768;
}

/// a chunk's blocks in the b1.7.3 layout, index x << 11 | z << 7 | y, with ids up to 4095 and 4 bit metadata
pub struct ChunkBlocks
{
    pub m_blocks: Vec<u16>,
    pub m_meta: Vec<u8>,
}

impl Default for ChunkBlocks
{
    fn default() -> Self { Self { m_blocks: vec![0; chunk::BLOCK_COUNT], m_meta: vec![0; chunk::BLOCK_COUNT] } }
}

impl ChunkBlocks
{
    pub fn index(x: i32, y: i32, z: i32) -> usize { (x << 11 | z << 7 | y) as usize }

    pub fn get(&self, x: i32, y: i32, z: i32) -> i32
    {
        if !(0..chunk::HEIGHT).contains(&y)
        {
            return 0;
        }

        self.m_blocks[Self::index(x, y, z)] as i32
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, block: i32) { self.setWithMeta(x, y, z, block, 0); }

    pub fn setWithMeta(&mut self, x: i32, y: i32, z: i32, block: i32, meta: i32)
    {
        if !(0..chunk::HEIGHT).contains(&y)
        {
            return;
        }

        let index: usize = Self::index(x, y, z);
        self.m_blocks[index] = block as u16;
        self.m_meta[index] = (meta & 15) as u8;
    }
}

/// net.minecraft.src.MathHelper, the table sine keeps ported generators drawing the same shapes
pub mod math
{
    use std::sync::OnceLock;

    const TABLE_SIZE: usize = 65536;
    const TABLE_SCALE: f32 = 10430.378;

    fn table() -> &'static [f32]
    {
        static TABLE: OnceLock<Vec<f32>> = OnceLock::new();

        TABLE.get_or_init(||
        {
            (0..TABLE_SIZE).map(|i| (i as f64 * std::f64::consts::PI * 2.0 / TABLE_SIZE as f64).sin() as f32).collect()
        })
    }

    pub fn sin(value: f32) -> f32 { table()[((value * TABLE_SCALE) as i32 & 0xFFFF) as usize] }

    pub fn cos(value: f32) -> f32 { table()[((value * TABLE_SCALE + 16384.0) as i32 & 0xFFFF) as usize] }

    // sqrt_double rounds to a float so every vanilla length and distance carries that rounding
    pub fn sqrt(value: f64) -> f32 { value.sqrt() as f32 }

    pub fn floor(value: f64) -> i32
    {
        let truncated: i32 = value as i32;
        if value < truncated as f64
        {
            truncated - 1
        }
        else
        {
            truncated
        }
    }
}
