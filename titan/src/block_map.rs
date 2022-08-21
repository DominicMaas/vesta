pub type BlockFace = usize;

pub const FACE_FRONT: BlockFace = 0;
pub const FACE_BACK: BlockFace = 1;
pub const FACE_RIGHT: BlockFace = 2;
pub const FACE_LEFT: BlockFace = 3;
pub const FACE_TOP: BlockFace = 4;
pub const FACE_BOTTOM: BlockFace = 5;

pub const TEX_X_STEP: f32 = 1.0 / 16.0;
pub const TEX_Y_STEP: f32 = 1.0;

pub fn vertex_offset(input: [f32; 3], x: f32, y: f32, z: f32) -> [f32; 3] {
    return [x + input[0], y + input[1], z + input[2]];
}

// Defined in counter clockwise
pub const VERTEX_MAP: [[[f32; 3]; 4]; 6] = [
    [
        // Front
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ],
    [
        //  Back
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
    ],
    [
        // Right
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
        [1.0, 0.0, 1.0],
    ],
    [
        // Left
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0],
    ],
    [
        // Top
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
    ],
    [
        // Bottom
        [1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
    ],
];

pub const NORMAL_MAP: [[[f32; 3]; 4]; 6] = [
    [
        // Front
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ],
    [
        // Back
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
    ],
    [
        // Right
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
    ],
    [
        // Left
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
    ],
    [
        // Top
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ],
    [
        // Bottom
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
    ],
];

pub const INDEX_MAP: [[u32; 6]; 6] = [
    [0, 1, 2, 2, 3, 0],
    [0, 1, 2, 2, 3, 0],
    [0, 1, 2, 2, 3, 0],
    [0, 1, 2, 2, 3, 0],
    [0, 1, 2, 2, 3, 0],
    [0, 1, 2, 2, 3, 0],
];
