use super::{Random, chunk};
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockPos, StructurePiece, StructureStart};

/// StructureBoundingBox from later versions, inclusive on both corners
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StructureBox
{
    pub m_min: [i32; 3],
    pub m_max: [i32; 3],
}

// MapGenStructure.generateStructuresInChunk builds walls from y 1 up through this height
const POPULATE_TOP: i32 = 512;
// a chunk populates the 16 by 16 columns offset half a chunk from its corner
const POPULATE_OFFSET: i32 = 8;

impl StructureBox
{
    pub const fn new(min: [i32; 3], max: [i32; 3]) -> StructureBox { StructureBox { m_min: min, m_max: max } }

    pub fn populated(chunkX: i32, chunkZ: i32) -> StructureBox
    {
        let x: i32 = chunkX * chunk::WIDTH + POPULATE_OFFSET;
        let z: i32 = chunkZ * chunk::WIDTH + POPULATE_OFFSET;
        StructureBox::new([x, 1, z], [x + chunk::WIDTH - 1, POPULATE_TOP, z + chunk::WIDTH - 1])
    }

    pub fn contains(&self, x: i32, y: i32, z: i32) -> bool
    {
        (self.m_min[0]..=self.m_max[0]).contains(&x)
            && (self.m_min[1]..=self.m_max[1]).contains(&y)
            && (self.m_min[2]..=self.m_max[2]).contains(&z)
    }

    pub fn intersects(&self, other: &StructureBox) -> bool
    {
        (0..3).all(|axis| self.m_max[axis] >= other.m_min[axis] && self.m_min[axis] <= other.m_max[axis])
    }

    pub fn offset(&mut self, by: [i32; 3])
    {
        for axis in 0..3
        {
            self.m_min[axis] += by[axis];
            self.m_max[axis] += by[axis];
        }
    }

    /// StructureBoundingBox.expandTo
    pub fn include(&mut self, other: &StructureBox)
    {
        for axis in 0..3
        {
            self.m_min[axis] = self.m_min[axis].min(other.m_min[axis]);
            self.m_max[axis] = self.m_max[axis].max(other.m_max[axis]);
        }
    }

    /// needs api 1.10, the box as a recorded piece with the mod's own name for it
    pub fn piece(&self, label: &str) -> StructurePiece
    {
        StructurePiece { min: self.m_min, max: self.m_max, label: label.to_owned() }
    }
}

/// needs api 1.10, a start of a kind at its origin whose box spans every piece, the origin alone when there are none
pub fn start(structure: &Identifier, origin: BlockPos, pieces: Vec<StructurePiece>) -> StructureStart
{
    let at: [i32; 3] = [origin.x, origin.y, origin.z];
    let mut bounds: StructureBox = StructureBox::new(at, at);

    if let Some(first) = pieces.first()
    {
        bounds = StructureBox::new(first.min, first.max);
    }

    for piece in &pieces
    {
        bounds.include(&StructureBox::new(piece.min, piece.max));
    }

    StructureStart { structure: structure.clone(), origin, min: bounds.m_min, max: bounds.m_max, pieces }
}

/// MapGenBase.generate and MapGenStructure.recursiveGenerate: every chunk in range of the generated one is seeded the
/// same way and draws one int before the structure decides whether it starts there
pub fn candidates(seed: i64, chunkX: i32, chunkZ: i32, range: i32, mut visit: impl FnMut(i32, i32, &Random))
{
    let random: Random = Random::withSeed(seed);
    let scales: [i64; 2] = [random.nextLong(), random.nextLong()];

    for startX in chunkX - range..=chunkX + range
    {
        for startZ in chunkZ - range..=chunkZ + range
        {
            reseed(&random, scales, seed, startX, startZ);
            visit(startX, startZ, &random);
        }
    }
}

/// needs api 1.10, the random candidates hands one chunk, so a structure finder asks the same question generation does
pub fn candidate(seed: i64, chunkX: i32, chunkZ: i32) -> Random
{
    let random: Random = Random::withSeed(seed);
    let scales: [i64; 2] = [random.nextLong(), random.nextLong()];
    reseed(&random, scales, seed, chunkX, chunkZ);
    random
}

fn reseed(random: &Random, [xScale, zScale]: [i64; 2], seed: i64, chunkX: i32, chunkZ: i32)
{
    random.setSeed((chunkX as i64).wrapping_mul(xScale) ^ (chunkZ as i64).wrapping_mul(zScale) ^ seed);
    random.nextInt();
}
