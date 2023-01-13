use super::read_utils::{read_bool, read_float, read_int, read_string};
use crate::replay::{assert_start_of_block, BlockType, ReplayFloat, ReplayInt, ReplayTime, Result};
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
    pub score: ReplayInt,
    pub mode: String,
    pub environment: String,
    pub modifiers: String,
    pub jump_distance: ReplayFloat,
    pub left_handed: bool,
    pub height: ReplayFloat,
    pub start_time: ReplayTime,
    pub fail_time: ReplayTime,
    pub speed: ReplayTime,
}

impl Info {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Info> {
        assert_start_of_block(r, BlockType::Info)?;

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
    use crate::replay::BsorError;
    use crate::tests_util::{append_info, generate_random_info};
    use std::io::Cursor;

    #[test]
    fn it_returns_invalid_bsor_error_when_info_block_id_is_invalid() -> Result<()> {
        let buf = Vec::from([255u8]);

        let result = Info::load(&mut Cursor::new(buf));

        assert!(matches!(result, Err(BsorError::InvalidBsor)));

        Ok(())
    }

    #[test]
    fn it_can_load_info() -> Result<()> {
        let info = generate_random_info();

        let info_id = BlockType::Info.try_into()?;
        let mut buf = Vec::from([info_id]);

        append_info(&mut buf, &info)?;

        let result = Info::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(result, info);

        Ok(())
    }
}
