use super::error::BsorError;
use super::read_utils::{read_bool, read_byte, read_float, read_int, read_string};
use std::io::Read;

#[derive(PartialEq, Debug)]
pub struct Info {
    pub version: String,
    pub game_version: String,
    pub timestamp: u32,
    pub player_id: String,
    pub player_name: String,
    pub platform: String,
    pub tracking_system: String,
    pub hmd: String,
    pub controller: String,
    pub hash: String,
    pub song_name: String,
    pub mapper: String,
    pub difficulty: String,
    pub score: i32,
    pub mode: String,
    pub environment: String,
    pub modifiers: String,
    pub jump_distance: f32,
    pub left_handed: bool,
    pub height: f32,
    pub start_time: f32,
    pub fail_time: f32,
    pub speed: f32,
}

impl Info {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Info, BsorError> {
        match read_byte(r) {
            Ok(v) => {
                if v != 0 {
                    return Err(BsorError::InvalidBsor);
                }
            }
            Err(e) => return Err(e),
        }

        let version = read_string(r)?;
        let game_version = read_string(r)?;
        let timestamp = read_string(r)?.parse()?;
        let player_id = read_string(r)?;
        let player_name = read_string(r)?;
        let platform = read_string(r)?;
        let tracking_system = read_string(r)?;
        let hmd = read_string(r)?;
        let controller = read_string(r)?;
        let hash = read_string(r)?;
        let song_name = read_string(r)?;
        let mapper = read_string(r)?;
        let difficulty = read_string(r)?;
        let score = read_int(r)?;
        let mode = read_string(r)?;
        let environment = read_string(r)?;
        let modifiers = read_string(r)?;
        let jump_distance = read_float(r)?;
        let left_handed = read_bool(r)?;
        let height = read_float(r)?;
        let start_time = read_float(r)?;
        let fail_time = read_float(r)?;
        let speed = read_float(r)?;

        Ok(Info {
            version,
            game_version,
            timestamp,
            player_id,
            player_name,
            platform,
            tracking_system,
            hmd,
            controller,
            hash,
            song_name,
            mapper,
            difficulty,
            score,
            mode,
            environment,
            modifiers,
            jump_distance,
            left_handed,
            height,
            start_time,
            fail_time,
            speed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_util::append_str;
    use std::io::Cursor;

    #[test]
    fn it_can_load_info() {
        let version = "0.5.4".to_owned();
        let game_version = "1.27.0".to_owned();
        let timestamp = "1662289178".to_owned();
        let player_id = "76561198035381239".to_owned();
        let player_name = "xor eax eax".to_owned();
        let platform = "steam".to_owned();
        let tracking_system = "Oculus".to_owned();
        let hmd = "Rift_S".to_owned();
        let controller = "Unknown".to_owned();
        let hash = "C3CFED196F96B161C0862EC387E0EE9241CD5B48".to_owned();
        let song_name = "Novablast".to_owned();
        let mapper = "Bitz".to_owned();
        let difficulty = "Expert".to_owned();
        let score = 1216422;
        let mode = "Standard".to_owned();
        let environment = "Timbaland".to_owned();
        let modifiers = "DA,FS".to_owned();
        let jump_distance = 19.96f32;
        let left_handed = false;
        let height = 1.76f32;
        let start_time = 0.0f32;
        let fail_time = 0.0f32;
        let speed = 0.0f32;

        let mut buf = Vec::from([0 as u8]);
        append_str(&mut buf, &version);
        append_str(&mut buf, &game_version);
        append_str(&mut buf, &timestamp);
        append_str(&mut buf, &player_id);
        append_str(&mut buf, &player_name);
        append_str(&mut buf, &platform);
        append_str(&mut buf, &tracking_system);
        append_str(&mut buf, &hmd);
        append_str(&mut buf, &controller);
        append_str(&mut buf, &hash);
        append_str(&mut buf, &song_name);
        append_str(&mut buf, &mapper);
        append_str(&mut buf, &difficulty);
        buf.append(&mut i32::to_le_bytes(score).to_vec());
        append_str(&mut buf, &mode);
        append_str(&mut buf, &environment);
        append_str(&mut buf, &modifiers);
        buf.append(&mut f32::to_le_bytes(jump_distance).to_vec());
        buf.append(&mut (if left_handed { [1] } else { [0] }).to_vec());
        buf.append(&mut f32::to_le_bytes(height).to_vec());
        buf.append(&mut f32::to_le_bytes(start_time).to_vec());
        buf.append(&mut f32::to_le_bytes(fail_time).to_vec());
        buf.append(&mut f32::to_le_bytes(speed).to_vec());

        let result = Info::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(
            result,
            Info {
                version,
                game_version,
                timestamp: timestamp.parse::<u32>().unwrap(),
                player_id,
                player_name,
                platform,
                tracking_system,
                hmd,
                controller,
                hash,
                song_name,
                mapper,
                difficulty,
                score,
                mode,
                environment,
                modifiers,
                jump_distance,
                left_handed,
                height,
                start_time,
                fail_time,
                speed,
            }
        );
    }
}
