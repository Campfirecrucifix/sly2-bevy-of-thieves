//! WP-C tests: every mesh of the 43 levels converts, every texture decodes; CLUT swizzle. Needs the ISO at
//! "<root>/Sly 2 - Band of Thieves (USA).iso" (level list from the boot level table).
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::{LevelLoader, LoadedLevel};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::path::PathBuf;
use std::sync::Arc;

fn real_root() -> PathBuf {
    let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if r.join("Sly 2 - Band of Thieves (USA).iso").exists() { r } else { PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) }
}

/// The 43 level descriptors (db disc_files + level_table, as in engine_a).
fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

fn load(loader: &LevelLoader, d: Descriptor) -> (LoadedLevel, World) {
    let lv = crate::common::load_blocking(&loader, d).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap_or_else(|e| panic!("lsn {}: {e}", d.lsn()));
    (lv, w)
}



use sha2::{Digest, Sha256};
use sly2_recomp::engine::gfx::mesh::strip_triangles;
use sly2_recomp::engine::gfx::scene::LevelScene;
use sly2_recomp::engine::gfx::texture::{csm1_index, decode, ps2_alpha, Bitmap, Clut, PSMT4, PSMT8};

#[test]
fn clut_csm1_swizzle() {
    // bits 3 and 4 swapped: 0..7 stay, 8..15 -> 16..23, 16..23 -> 8..15
    assert_eq!(csm1_index(0), 0);
    assert_eq!(csm1_index(7), 7);
    assert_eq!(csm1_index(8), 16);
    assert_eq!(csm1_index(16), 8);
    assert_eq!(csm1_index(0x18), 0x18);
    assert_eq!(csm1_index(0xff), 0xff);
    assert_eq!(csm1_index(0x2a), 0x32);
    let perm: std::collections::BTreeSet<usize> = (0..=255u8).map(csm1_index).collect();
    assert_eq!(perm.len(), 256);
    // T8 with a palette where entry k = k: pixel 8 must read stored entry 16
    let clut = Clut { id: 0, kind: 0, colors: 256, rgba: Some((0..256).map(|k| [k as u8, 0, 0, 0x80]).collect()) };
    let bmp = Bitmap { id: 0, kind: 0, width: 4, height: 1, psm: PSMT8, tbw: 1, pixels: Some(vec![0, 8, 16, 31]) };
    let t = decode(&bmp, Some(&clut)).unwrap();
    assert_eq!(t.rgba.chunks(4).map(|p| p[0]).collect::<Vec<_>>(), vec![0, 16, 8, 31]);
    assert_eq!(t.rgba[3], 255); // alpha 0x80 = 1.0
    // T4: low nibble first, linear 8x2 palette
    let clut = Clut { id: 0, kind: 0, colors: 16, rgba: Some((0..16).map(|k| [k as u8, 0, 0, 0x40]).collect()) };
    let bmp = Bitmap { id: 0, kind: 0, width: 2, height: 1, psm: PSMT4, tbw: 1, pixels: Some(vec![0x9a]) };
    let t = decode(&bmp, Some(&clut)).unwrap();
    assert_eq!((t.rgba[0], t.rgba[4], t.rgba[3]), (10, 9, 128));
    assert_eq!(ps2_alpha(0xff), 255);
    assert_eq!(ps2_alpha(0), 0);
}

#[test]
fn strip_adc() {
    let mut t = Vec::new();
    strip_triangles(&[0, 1, 2, 3, 4, 5], &[0b10011], 0, 6, &mut t);
    // bits 0,1,4 set: k=2 even (0,1,2); k=3 odd (2,1,3); k=4 ADC skipped; k=5 odd (4,3,5)
    assert_eq!(t, vec![[0, 1, 2], [2, 1, 3], [4, 3, 5]]);
}

/// (lsn, objects, prims, vertices, triangles, faces agreeing with stored normals, textures decoded,
/// texture errors, sha256 prefix of all decoded RGBA)
type Row = (u32, usize, usize, usize, usize, usize, usize, usize, String);
// WP-C4 re-bless: type 1/2 records now convert to the faces their edge quads reference (was: positions as sequential strips).
const EXPECT: &[(u32, usize, usize, usize, usize, usize, usize, usize, &str)] = &[
    // re-blessed after WP-C4: mesh types 1/2/3 now converted into LevelScene (+objects/meshes; texture hashes unchanged)
    (335488, 421, 17335, 666511, 321888, 320562, 1257, 0, "8767326da5947a2d"),
    (350144, 2, 6, 122, 57, 57, 4, 0, "54f8e70d6aae0e4e"),
    (353856, 439, 13185, 518824, 243861, 241941, 1578, 0, "f3958ff3dd4bc0bd"),
    (377568, 246, 6677, 263702, 130369, 129705, 923, 0, "100bc152130791b4"),
    (388768, 295, 8471, 355758, 164840, 164058, 1243, 0, "1f8af8a6d349bf86"),
    (402976, 237, 5865, 246008, 123079, 122468, 877, 0, "b2049b08983d638b"),
    (409312, 200, 3345, 137663, 76096, 75482, 748, 0, "d30484fbbd9957c2"),
    (417504, 203, 5086, 164219, 93934, 93300, 693, 0, "9b7de8612424f8ac"),
    (427328, 385, 9720, 429753, 218583, 217544, 1322, 0, "cd5080ebf67acf69"),
    (444896, 233, 7628, 326005, 166056, 165311, 1061, 0, "95485cfdbeeb96c3"),
    (453760, 227, 4235, 165838, 91476, 90751, 901, 0, "8c11b1f16221bd9f"),
    (465056, 202, 4303, 176302, 108834, 107920, 717, 0, "0940ae8c5f979afb"),
    (482752, 438, 10076, 429677, 222081, 220991, 1510, 0, "87bd242a3eaba27a"),
    (509056, 314, 7250, 293097, 142948, 142299, 1062, 0, "be6273316d0fd8d8"),
    (521344, 456, 13526, 578716, 248396, 247055, 1797, 0, "221f8d7f8e973e5b"),
    (540160, 294, 7527, 320284, 147388, 146596, 1053, 0, "4f9cbdd91a649329"),
    (552768, 124, 1906, 79762, 49860, 49383, 455, 0, "42bd163e115c6af9"),
    (561312, 434, 11560, 464693, 227020, 225805, 1390, 0, "05b40fb884ba8344"),
    (578848, 142, 2565, 101616, 62267, 61716, 534, 0, "403814cfa584897d"),
    (589920, 322, 6266, 246746, 136007, 135077, 1007, 0, "39dc3e298bf62e0c"),
    (603648, 171, 3733, 153084, 86830, 86084, 688, 0, "3e38a1b634619fd1"),
    (615360, 180, 2929, 126748, 72886, 72257, 561, 0, "64e644821d076817"),
    (626688, 152, 2378, 99879, 60609, 59999, 538, 0, "5f5e0113559204ae"),
    (637920, 161, 2293, 96599, 59616, 59060, 505, 0, "bbf730b97a7c89f1"),
    (649056, 155, 2569, 107658, 64009, 63402, 543, 0, "e7800e411d1228ea"),
    (660320, 150, 2180, 88379, 55289, 54741, 518, 0, "8c8bebc77158c4a8"),
    (671392, 350, 9684, 369267, 197270, 196137, 1262, 0, "f2ad7364adb2c0b7"),
    (687488, 217, 3448, 133501, 77591, 76946, 698, 0, "76818bd68673f24a"),
    (698912, 263, 6668, 289882, 158215, 157502, 915, 0, "511789168e613f6a"),
    (710528, 276, 7484, 312673, 167866, 167092, 955, 0, "7e77d63e02eb9dd2"),
    (720608, 230, 7982, 311590, 169471, 168760, 820, 0, "865c7fa01854aed9"),
    (730464, 427, 11162, 412634, 203692, 202665, 1618, 0, "f96a807fb06a2f65"),
    (749312, 195, 4026, 143511, 79289, 78728, 683, 0, "589587b3a6060056"),
    (761664, 192, 4849, 189225, 97456, 96918, 733, 0, "3a9833a05aad8e49"),
    (773120, 196, 4149, 141119, 78879, 78322, 685, 0, "98966cbcdd97e488"),
    (784544, 138, 1946, 78939, 51003, 50458, 393, 0, "2d53a8f431b4212c"),
    (795296, 167, 3706, 134111, 77633, 77051, 621, 0, "16d9a9fc9f8db1a8"),
    (808192, 458, 13683, 446382, 210659, 209516, 1702, 0, "9f2d834bc4d059a1"),
    (823200, 285, 5258, 206221, 112593, 111595, 885, 0, "317878fff47583a7"),
    (834976, 175, 2614, 108741, 61140, 60621, 587, 0, "30d4a18164b9de03"),
    (845728, 197, 3162, 128143, 72163, 71622, 704, 0, "b8a5a108ef32c462"),
    (856896, 177, 2711, 114257, 63031, 62535, 573, 0, "582f7f84288c3696"),
    (867648, 470, 7927, 354797, 169980, 168111, 1431, 0, "98203d76f8bf40a5"),
];

#[test]
#[ignore = "heavy"]
fn all_levels_convert_and_decode() {
    let (loader, descs) = levels();
    let mut rows: Vec<Row> = Vec::new();
    let expect: Vec<Row> = EXPECT.iter().map(|r| (r.0, r.1, r.2, r.3, r.4, r.5, r.6, r.7, r.8.to_string())).collect();
    let mut psm = std::collections::BTreeMap::new();
    for d in descs {
        let (lv, w) = load(&loader, d);
        let sc = LevelScene::build(&lv, &w);
        let (mut np, mut nv, mut nt, mut agree) = (0, 0, 0, 0);
        for o in &sc.objects {
            for p in &o.prims {
                let m = &p.prim;
                np += 1;
                nv += m.positions.len();
                nt += m.indices.len() / 3;
                let n = m.positions.len();
                assert!(m.indices.len() % 3 == 0 && !m.indices.is_empty());
                assert!(m.indices.iter().all(|&i| (i as usize) < n), "lsn {} index out of range", d.lsn());
                assert!(m.normals.len() == n && m.uvs.len() == n && m.colors.len() == n && m.tangents.len() == n);
                let fin = |v: &[f32]| v.iter().all(|x| x.is_finite());
                assert!(m.positions.iter().all(|v| fin(v)) && m.normals.iter().all(|v| fin(v)) && m.uvs.iter().all(|v| fin(v)));
                assert!(m.tangents.iter().all(|v| fin(v)) && m.colors.iter().all(|v| fin(v)));
                assert!(o.local.iter().chain(p.local.iter()).all(|r| fin(r)));
                for t in m.indices.chunks(3) {
                    let [a, b, c] = [0, 1, 2].map(|k| m.positions[t[k] as usize]);
                    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                    let f = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                    let sn = [0, 1, 2].map(|j| (0..3).map(|k| m.normals[t[k] as usize][j]).sum::<f32>());
                    if f[0] * sn[0] + f[1] * sn[1] + f[2] * sn[2] > 0.0 {
                        agree += 1;
                    }
                }
            }
        }
        let (mut ok, mut bad) = (0, 0);
        let mut h = Sha256::new();
        for et in &sc.textures {
            for b in &et.bitmaps {
                *psm.entry(b.psm).or_insert(0) += 1;
            }
            for s in 0..et.shaders.len() {
                for f in 0..et.shaders[s].frames.len() {
                    match et.decode_shader(s, f) {
                        Some(Ok(t)) => {
                            assert_eq!(t.rgba.len(), (t.width * t.height * 4) as usize);
                            h.update(&t.rgba);
                            ok += 1;
                        }
                        Some(Err(e)) => {
                            if bad < 3 { println!("lsn {} shader {s} frame {f}: {e:?}", d.lsn()); }
                            bad += 1
                        }
                        None => bad += 1,
                    }
                }
            }
        }
        let hx: String = h.finalize().iter().take(8).map(|b| format!("{b:02x}")).collect();
        rows.push((d.lsn(), sc.objects.len(), np, nv, nt, agree, ok, bad, hx));
    }
    println!("psm histogram {psm:?}");
    if std::env::var("ENGINE_C_BLESS").is_ok() || EXPECT.is_empty() {
        for r in &rows { println!("BLESS     {r:?},"); }
        assert!(!EXPECT.is_empty(), "bless EXPECT");
        return;
    }
    assert_eq!(rows, expect);
}
