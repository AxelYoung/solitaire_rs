use wgpu::util::DeviceExt;

use crate::systems::{Vec2, GameState, SCREEN_SIZE, Quad, Stack};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2]
}

const SPRITE_COUNT: [u8; 2] = [13, 5];

const QUAD_VERTS: [Vertex; 4] =  [
    Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0], }, // Top right
    Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0], }, // Top left
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0], }, // Bottom left
    Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0], }, // Bottom right
];

const QUAD_INDIS: [u16; 6] = [
    0, 1, 2,
    0, 2, 3
];

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2
                }
            ]
        }
    }
}

pub fn create_buffers(device: &wgpu::Device, state: &GameState) -> (Option<wgpu::Buffer>, Option<wgpu::Buffer>, usize) {

    let mut verts : Vec<Vertex> = vec![];
    let mut indis : Vec<u16> = vec![];

    create_quad(&state.stock.quad, stack_index(&state.stock), &mut verts, &mut indis);

    for stack in state.tableau.iter() {
        create_quad(&stack.quad, stack_index(&stack), &mut verts, &mut indis);
    }

    for stack in state.foundations.iter() {
        create_quad(&stack.quad, stack_index(&stack), &mut verts, &mut indis);
    }

    let vertex_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX
        }
    );

    let index_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indis),
            usage: wgpu::BufferUsages::INDEX
        }
    );

    (Some(vertex_buffer), Some(index_buffer), indis.len())
}

fn stack_index(stack: &Stack) -> [u8; 2] {
    if stack.cards.len() == 0 {
        [1, 4]
    } else if stack.top_hidden {
        [0, 4]
    } else {
        index_from_card(stack.cards[0])
    }
}

fn index_from_card(card: u8) -> [u8; 2] {
    [card % 12, card / 12]
}

fn create_quad(quad: &Quad, sprite_index: [u8; 2], verts: &mut Vec<Vertex>, indis: &mut Vec<u16>) {
    let mut tile_verts : Vec<Vertex> = QUAD_VERTS.iter()
        .map(|v| Vertex {
            position: { 
                [((quad.pos.x + v.position[0] * quad.size.x as f32) / SCREEN_SIZE.x as f32), 
                ((quad.pos.y + v.position[1] * quad.size.y as f32) / SCREEN_SIZE.y as f32), 
                v.position[2]]
            },
            tex_coords: uv_from_index(v.tex_coords, sprite_index)
        })
        .collect();

    let mut tile_indis : Vec<u16> = QUAD_INDIS.iter()
        .map(|i| i + verts.len() as u16)
        .collect();

    verts.append(&mut tile_verts);
    indis.append(&mut tile_indis);
}

fn uv_from_index(uv: [f32; 2], sprite_index: [u8; 2]) -> [f32; 2] {
    return [
        uv[0] / SPRITE_COUNT[0] as f32 + (sprite_index[0] as f32 / SPRITE_COUNT[0] as f32),
        uv[1] / SPRITE_COUNT[1] as f32 + (sprite_index[1] as f32 / SPRITE_COUNT[1] as f32),
    ]
}