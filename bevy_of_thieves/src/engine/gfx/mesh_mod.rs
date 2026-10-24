//! RENDER: mesh override mods (port addition, no PS2 equivalent).
//! `mods/<mod>/meshes/<level lsn>/<object index>_<record>.obj` replaces the geometry of one static
//! (non-skinned) surface prim; the prim keeps its material, transform and fog. Wavefront OBJ subset:
//! `v`, `vt`, `vn`, `f` (triangles or polygons, fan-triangulated; `v`, `v/t`, `v//n`, `v/t/n`,
//! negative indices). Coordinates are in the prim's local frame (same units as the original).

/// Mod-relative path of a prim mesh override.
/// `object` = index in the level's object list (`World::objects`, stable for a given level file; placed
/// template copies share their source object's slot), `record` = the ALO mesh record of the prim.
pub fn mesh_override_path(lsn: u32, object: usize, record: usize) -> String {
    format!("meshes/{lsn}/{object}_{record}.obj")
}

/// Flat triangle list (3 vertices per triangle, unindexed).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ObjMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
}

pub fn parse_obj(text: &str) -> Result<ObjMesh, String> {
    let (mut v, mut vt, mut vn) = (Vec::<[f32; 3]>::new(), Vec::<[f32; 2]>::new(), Vec::<[f32; 3]>::new());
    let mut out = ObjMesh::default();
    let nums = |it: std::str::SplitWhitespace, n: usize, line: usize| -> Result<Vec<f32>, String> {
        let r: Vec<f32> = it.take(n).map(|s| s.parse::<f32>()).collect::<Result<_, _>>().map_err(|e| format!("obj:{line}: {e}"))?;
        if r.len() < n.min(2) {
            return Err(format!("obj:{line}: too few numbers"));
        }
        Ok(r)
    };
    for (ln, l) in text.lines().enumerate() {
        let mut it = l.split_whitespace();
        match it.next() {
            Some("v") => {
                let r = nums(it, 3, ln + 1)?;
                v.push([r[0], r[1], *r.get(2).unwrap_or(&0.0)]);
            }
            Some("vt") => {
                let r = nums(it, 2, ln + 1)?;
                // OBJ v=0 is the bottom; Bevy/PS2 v=0 is the top
                vt.push([r[0], 1.0 - r[1]]);
            }
            Some("vn") => {
                let r = nums(it, 3, ln + 1)?;
                vn.push([r[0], r[1], *r.get(2).unwrap_or(&0.0)]);
            }
            Some("f") => {
                let idx = |s: &str, len: usize| -> Result<Option<usize>, String> {
                    if s.is_empty() {
                        return Ok(None);
                    }
                    let i: i64 = s.parse().map_err(|e| format!("obj:{}: {e}", ln + 1))?;
                    let i = if i < 0 { len as i64 + i } else { i - 1 };
                    (0..len as i64).contains(&i).then_some(Some(i as usize)).ok_or(format!("obj:{}: index out of range", ln + 1))
                };
                let mut poly = Vec::new();
                for c in it {
                    let mut p = c.split('/');
                    let pi = idx(p.next().unwrap_or(""), v.len())?.ok_or(format!("obj:{}: face without vertex", ln + 1))?;
                    let ti = idx(p.next().unwrap_or(""), vt.len())?;
                    let ni = idx(p.next().unwrap_or(""), vn.len())?;
                    poly.push((pi, ti, ni));
                }
                for k in 1..poly.len().saturating_sub(1) {
                    for &(pi, ti, ni) in &[poly[0], poly[k], poly[k + 1]] {
                        out.positions.push(v[pi]);
                        out.uvs.push(ti.map_or([0.0, 0.0], |t| vt[t]));
                        out.normals.push(ni.map_or([0.0; 3], |n| vn[n]));
                    }
                }
            }
            _ => {}
        }
    }
    if out.positions.is_empty() {
        return Err("obj: no faces".into());
    }
    // missing normals -> flat face normals
    for t in (0..out.positions.len()).step_by(3) {
        if out.normals[t..t + 3].iter().all(|n| *n == [0.0; 3]) {
            let [a, b, c] = [out.positions[t], out.positions[t + 1], out.positions[t + 2]];
            let (e1, e2) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [c[0] - a[0], c[1] - a[1], c[2] - a[2]]);
            let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-12);
            out.normals[t..t + 3].fill([n[0] / l, n[1] / l, n[2] / l]);
        }
    }
    Ok(out)
}

/// Bevy mesh of an OBJ override (white vertex colour: the original baked lighting does not apply).
#[cfg(feature = "game")]
pub fn to_bevy_mesh(o: &ObjMesh) -> bevy::prelude::Mesh {
    use bevy::prelude::*;
    use bevy::render::render_resource::PrimitiveTopology;
    let n = o.positions.len() as u32;
    // U32 indices + tangents: same attribute set as gfx::game::to_bevy_mesh (Solari-compatible)
    let mut m = Mesh::new(PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, o.positions.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, o.normals.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, o.uvs.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32, 1.0, 1.0, 1.0]; o.positions.len()])
        .with_inserted_indices(bevy::mesh::Indices::U32((0..n).collect()));
    if m.generate_tangents().is_err() {
        m.insert_attribute(Mesh::ATTRIBUTE_TANGENT, vec![[1.0f32, 0.0, 0.0, 1.0]; n as usize]);
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn obj_quad_fan_and_flat_normals() {
        let o = parse_obj("v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nvt 0 0\nf 1/1 2/1 3/1 -1/1\n").unwrap();
        assert_eq!(o.positions.len(), 6);
        assert_eq!(o.normals[0], [0.0, 0.0, 1.0]);
        assert_eq!(o.uvs[0], [0.0, 1.0]);
        assert!(parse_obj("v 0 0 0\nf 1 2 3\n").is_err());
        assert_eq!(mesh_override_path(353856, 12, 0), "meshes/353856/12_0.obj");
    }
}
