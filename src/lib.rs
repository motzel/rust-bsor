/// Read and parse BS Open Replay (bsor) files
///
/// <https://github.com/BeatLeader/BS-Open-Replay>
///
/// # Examples
/// ```no_run
/// use bsor::replay::Replay;
/// use std::fs::File;
/// use std::io::BufReader;
///
/// let br = &mut BufReader::new(File::open("example.bsor").unwrap());
/// let replay = Replay::load(br).unwrap();
/// println!("{:#?}", replay);
/// ```
pub mod replay;

#[cfg(test)]
pub(crate) mod tests_util {
    use crate::replay::vector::{Vector3, Vector4};
    use rand::random;

    pub(crate) fn append_str(vec: &mut Vec<u8>, str: &str) {
        let len = str.len() as i32;
        vec.append(&mut i32::to_le_bytes(len).to_vec());
        vec.append(&mut str.as_bytes().to_vec());
    }

    pub(crate) fn generate_random_vec3() -> Vector3 {
        Vector3 {
            x: random::<f32>(),
            y: random::<f32>(),
            z: random::<f32>(),
        }
    }

    pub(crate) fn generate_random_vec4() -> Vector4 {
        Vector4 {
            x: random::<f32>(),
            y: random::<f32>(),
            z: random::<f32>(),
            w: random::<f32>(),
        }
    }
}
