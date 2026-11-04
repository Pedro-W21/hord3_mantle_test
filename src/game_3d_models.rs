use std::{collections::HashSet, f32::consts::PI, sync::Arc};

use hord3::{defaults::default_rendering::mantle::{api::ApiLod, shapes_to_tris::{cylinder_to_render_comp, vec_to_complex}}, horde::geometry::{rotation::Rotation, shapes_3d::{Cylinder, Sphere, Square}, vec3d::{Vec3D, Vec3Df}}};

use crate::tile_editor::nth_texture;

pub fn line_3D_from_to(start:Vec3Df, stop:Vec3Df, texture:String, side:f32, field:u32, light:(u8,u8,u8)) -> ApiLod {
    let (yaw, pitch) = start.get_orient_vers(&stop);
    let cylinder = Cylinder::<4>::new(Square::new_square(side) + start, start.dist(&stop)).rotate_around_barycenter(&Rotation::new_from_euler(PI/4.0, 0.0, 0.0)).rotate_around_base_center(&Rotation::new_from_euler(yaw, pitch, 0.0));
    cylinder_to_render_comp(&cylinder, texture.clone(), texture.clone(), texture.clone(), false, &vec![light ; 4], &vec![light ; 4], field, field, field, true, true)
}
pub fn simple_line(start:Vec3Df, stop:Vec3Df, texture:String, light:(u8,u8,u8)) -> ApiLod {
    let side = 0.25;
    let mut mesh = line_3D_from_to(
        start,
        stop,
        texture, side, 0, light);
    mesh
}

pub fn lit_selection_cube(start:Vec3Df, stop:Vec3Df, texture:String, light:(u8,u8,u8)) -> ApiLod {
    let side = 0.25;
    let mut mesh = line_3D_from_to(
        start,
        Vec3Df::new(stop.x, start.y, start.z),
        texture.clone(), side, 0, light);
    mesh.merge_with(line_3D_from_to(
        start,
        Vec3Df::new(stop.x, start.y, start.z),
        texture.clone(), side, 0, light));
    
    mesh.merge_with(line_3D_from_to(
        start,
        Vec3Df::new(start.x, stop.y, start.z),
        texture.clone(), side, 0, light));
    mesh.merge_with(line_3D_from_to(
        start,
        Vec3Df::new(start.x, start.y, stop.z),
        texture.clone(), side, 0, light));
    mesh.merge_with(line_3D_from_to(
        start,
        Vec3Df::new(start.x, start.y, stop.z),
        texture.clone(), side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(start.x, stop.y, stop.z),
        stop,
        texture.clone(), side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(stop.x, start.y, stop.z),
        stop,
        texture.clone(), side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(stop.x, stop.y, start.z),
        stop,
        texture.clone(), side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(stop.x, start.y, start.z),
        Vec3Df::new(stop.x, start.y, stop.z),
        texture.clone(), side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(start.x, stop.y, start.z),
        Vec3Df::new(start.x, stop.y, stop.z),
        texture.clone(), side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(stop.x, start.y, start.z),
        Vec3Df::new(stop.x, stop.y, start.z),
        texture.clone(), side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(start.x, stop.y, start.z),
        Vec3Df::new(stop.x, stop.y, start.z),
        texture.clone(), side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(start.x, start.y, stop.z),
        Vec3Df::new(stop.x, start.y, stop.z),
        texture.clone(), side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(start.x, start.y, stop.z),
        Vec3Df::new(start.x, stop.y, stop.z),
        texture.clone(), side, 0, light));
    mesh
}

pub fn selection_cube(start:Vec3Df, stop:Vec3Df, texture:String) -> ApiLod {
    lit_selection_cube(start, stop, texture, (255,255,255))
}

/*pub fn model_to_wireframe(model:ApiLod) -> ApiLod {
    let mut mesh = ApiLod::new(Vec::new(), Vec::new(), Vec::new());

    let mut in_set = HashSet::with_capacity(1024);


    for i in 0..model.triangles.len() {
        let tri_indices = model.triangles.get_indices_for_triangle(i);
        let tri = model.triangles.get_triangle(&model.x, &model.y, &model.z, i);
        if !in_set.contains(&(tri_indices[0], tri_indices[1])) && !in_set.contains(&(tri_indices[1], tri_indices[0])) {
            mesh.merge_with(
                line_3D_from_to(
                    Vec3Df::new(tri.p1.pos.x, tri.p1.pos.y, tri.p1.pos.z),
                    Vec3Df::new(tri.p2.pos.x, tri.p2.pos.y, tri.p2.pos.z),
                    7, 0.1, 0, (255,255,255))
            );
            in_set.insert((tri_indices[0], tri_indices[1]));
        }
        if !in_set.contains(&(tri_indices[1], tri_indices[2])) && !in_set.contains(&(tri_indices[2], tri_indices[1])) {
            mesh.merge_with(
                line_3D_from_to(
                    Vec3Df::new(tri.p2.pos.x, tri.p2.pos.y, tri.p2.pos.z),
                    Vec3Df::new(tri.p3.pos.x, tri.p3.pos.y, tri.p3.pos.z),
                    7, 0.1, 0, (255,255,255))
            );
            in_set.insert((tri_indices[1], tri_indices[2]));
        }
        if !in_set.contains(&(tri_indices[2], tri_indices[0])) && !in_set.contains(&(tri_indices[0], tri_indices[2])) {
            mesh.merge_with(
                line_3D_from_to(
                    Vec3Df::new(tri.p1.pos.x, tri.p1.pos.y, tri.p1.pos.z),
                    Vec3Df::new(tri.p3.pos.x, tri.p3.pos.y, tri.p3.pos.z),
                    7, 0.1, 0, (255,255,255))
            );
            in_set.insert((tri_indices[2], tri_indices[0]));
        } 
    }

    mesh
}*/

pub fn xyz_axis() -> ApiLod {
    // Texture sets 4,5,6
    let start = Vec3Df::zero();
    let side = 0.25;
    let mut mesh = line_3D_from_to(
        start,
        Vec3Df::new(1.0,0.0,0.0),
        nth_texture(4), side, 0, (255,255,255));
    mesh.merge_with(line_3D_from_to(
        start,
        Vec3Df::new(0.0,1.0,0.0),
        nth_texture(5), side, 0, (255,255,255)));
    
    mesh.merge_with(line_3D_from_to(
        start,
        Vec3Df::new(0.0,0.0,1.0),
        nth_texture(6), side, 0, (255,255,255)));
    mesh
}

pub fn xyz_mesh() -> ApiLod {
    xyz_axis()
}

pub fn sphere_mesh() -> ApiLod {
    let sphere = Sphere::new(Vec3D::zero(), 2.0).get_triangles::<4>(false);
    let tris = vec_to_complex(&sphere, vec![nth_texture(8)], &vec![0 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]);
    tris
}

/*pub fn wireframe_sphere_mesh() -> ApiLod {
    let sphere = Sphere::new(Vec3D::zero(), 2.0).get_triangles::<4>(true);
    let tris = model_to_wireframe(vec_to_complex(&sphere, &vec![8 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]));
    tris
}*/

pub fn textured_sphere_mesh() -> ApiLod {
    let sphere = Sphere::new(Vec3D::zero(), 2.0).get_triangles::<4>(false);
    let tris = vec_to_complex(&sphere, vec![nth_texture(2)], &vec![0 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]);
    tris
}

pub fn grey_sphere_mesh() -> ApiLod {
    let sphere = Sphere::new(Vec3D::zero(), 1.0).get_triangles::<4>(true);
    let mut tris = vec_to_complex(&sphere, vec![nth_texture(5)], &vec![0 ; sphere.len()], &vec![[(128,128,128) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]);
    
    tris
        
}


pub fn brown_sphere_mesh() -> ApiLod {
    let sphere = Sphere::new(Vec3D::zero(), 1.0).get_triangles::<4>(true);
    let mut tris = vec_to_complex(&sphere, vec![nth_texture(4)], &vec![0 ; sphere.len()], &vec![[(128,128,128) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]);
    
    tris
        
}

pub fn simple_projectile() -> ApiLod {
    let sphere = Sphere::new(Vec3D::zero(), 0.25).get_triangles::<4>(true);
    let mut tris = vec_to_complex(&sphere, vec![nth_texture(3)], &vec![0 ; sphere.len()], &vec![[(128,128,128) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]);
    
    tris
        
}

pub fn cannon_barrel() -> ApiLod {
    let barrel = line_3D_from_to(Vec3Df::zero(), Vec3Df::new(2.0, 0.0, 0.0), nth_texture(6), 0.15, 0, (255,255,255));
    barrel
}

pub fn factory() -> ApiLod {
    let barrel = line_3D_from_to(Vec3Df::zero(), Vec3Df::new(0.0, 0.0, 2.0), nth_texture(6), 1.0, 0, (255,255,255));
    barrel
}