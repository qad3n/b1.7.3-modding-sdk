use super::{Random, chunk};

// StructureBoundingBox from later versions, inclusive on both corners
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
        (self.m_min[0]..=self.m_max[0]).contains(&x) && (self.m_min[1]..=self.m_max[1]).contains(&y) && (self.m_min[2]..=self.m_max[2]).contains(&z)
    }

    pub fn intersects(&self, other: &StructureBox) -> bool { (0..3).all(|axis| self.m_max[axis] >= other.m_min[axis] && self.m_min[axis] <= other.m_max[axis]) }

    pub fn offset(&mut self, by: [i32; 3])
    {
        for axis in 0..3
        {
            self.m_min[axis] += by[axis];
            self.m_max[axis] += by[axis];
        }
    }

    // StructureBoundingBox.expandTo
    pub fn include(&mut self, other: &StructureBox)
    {
        for axis in 0..3
        {
            self.m_min[axis] = self.m_min[axis].min(other.m_min[axis]);
            self.m_max[axis] = self.m_max[axis].max(other.m_max[axis]);
        }
    }
}

// MapGenBase.generate and MapGenStructure.recursiveGenerate: every chunk in range of the generated one is seeded the
// same way and draws one int before the structure decides whether it starts there
pub fn candidates(seed: i64, chunkX: i32, chunkZ: i32, range: i32, mut visit: impl FnMut(i32, i32, &Random))
{
    let random: Random = Random::withSeed(seed);
    let xScale: i64 = random.nextLong();
    let zScale: i64 = random.nextLong();
    for startX in chunkX - range..=chunkX + range
    {
        for startZ in chunkZ - range..=chunkZ + range
        {
            random.setSeed((startX as i64).wrapping_mul(xScale) ^ (startZ as i64).wrapping_mul(zScale) ^ seed);
            random.nextInt();
            visit(startX, startZ, &random);
        }
    }
}
