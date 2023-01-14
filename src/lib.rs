pub mod prelude;
/// Read and parse BS Open Replay (bsor) files
///
/// <https://github.com/BeatLeader/BS-Open-Replay>
///
/// # Examples
/// Loading the entire replay into memory:
/// ```no_run
/// use bsor::prelude::*;
/// use std::fs::File;
/// use std::io::BufReader;
///
/// let br = &mut BufReader::new(File::open("example.bsor").unwrap());
/// let replay = Replay::load(br).unwrap();
/// println!("{:#?}", replay);
/// ```
///
/// Since you may rarely need the full replay structure (especially Frames block) and at the same time would like to keep memory usage low, there is also the option of loading only selected blocks (keep in mind that Header and Info blocks are always loaded)
///
/// ```no_run
/// use bsor::prelude::*;
/// use std::fs::File;
/// use std::io::BufReader;
///
/// let mut br = &mut BufReader::new(File::open("example.bsor").unwrap());
///
/// let parsed_replay = ParsedReplay::parse (br).unwrap();
///
/// let notes = parsed_replay.notes.load(br).unwrap();
/// println!(
///     "Info: {:#?}\nNotes count: {:#?}",
///     parsed_replay.info,
///     notes.len()
/// );
/// if !notes.is_empty() {
///     println!("{:#?}", notes.get_vec()[notes.len() / 2]);
/// }
/// ```
///
/// The memory savings can be significant, for example, for an **average replay of 1375kB**:
///
/// | Block         | Memory usage |
/// |---------------|--------------|
/// | Whole replay  | 1383kB       |
/// | Header + Info | 9kB          |
/// | Frames        | 1255kB       |
/// | Notes         | 137kB        |
pub mod replay;

#[cfg(test)]
pub(crate) mod tests_util {
    use crate::replay::frame::{Frame, Frames, PositionAndRotation};
    use crate::replay::height::{Height, Heights};
    use crate::replay::info::Info;
    use crate::replay::note::{
        ColorType, CutDirection, Note, NoteCutInfo, NoteEventType, NoteScoringType, Notes,
    };
    use crate::replay::pause::{Pause, Pauses};
    use crate::replay::wall::{Wall, Walls};
    use crate::replay::BSOR_MAGIC;
    use crate::replay::{
        vector::{Vector3, Vector4},
        BlockType, Replay, ReplayFloat, ReplayInt, ReplayLong,
    };
    use crate::replay::{ReplayTime, Result};
    use rand::random;

    pub(crate) fn append_str(vec: &mut Vec<u8>, str: &str) {
        let len = str.len() as i32;
        vec.append(&mut i32::to_le_bytes(len).to_vec());
        vec.append(&mut str.as_bytes().to_vec());
    }

    pub(crate) fn append_vector3(vec: &mut Vec<u8>, v3: &Vector3) {
        vec.append(&mut ReplayFloat::to_le_bytes(v3.x).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(v3.y).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(v3.z).to_vec());
    }

    pub(crate) fn append_vector4(vec: &mut Vec<u8>, v4: &Vector4) {
        vec.append(&mut ReplayFloat::to_le_bytes(v4.x).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(v4.y).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(v4.z).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(v4.w).to_vec());
    }

    pub(crate) fn append_position_and_rotation(vec: &mut Vec<u8>, pr: &PositionAndRotation) {
        append_vector3(vec, &pr.position);
        append_vector4(vec, &pr.rotation);
    }

    pub(crate) fn append_info(vec: &mut Vec<u8>, info: &Info) -> Result<()> {
        append_str(vec, &info.version);
        append_str(vec, &info.game_version);
        append_str(vec, &info.timestamp.to_string());
        append_str(vec, &info.player_id);
        append_str(vec, &info.player_name);
        append_str(vec, &info.platform);
        append_str(vec, &info.tracking_system);
        append_str(vec, &info.hmd);
        append_str(vec, &info.controller);
        append_str(vec, &info.hash);
        append_str(vec, &info.song_name);
        append_str(vec, &info.mapper);
        append_str(vec, &info.difficulty);
        vec.append(&mut ReplayInt::to_le_bytes(info.score).to_vec());
        append_str(vec, &info.mode);
        append_str(vec, &info.environment);
        append_str(vec, &info.modifiers);
        vec.append(&mut ReplayFloat::to_le_bytes(info.jump_distance).to_vec());
        vec.append(&mut (if info.left_handed { [1] } else { [0] }).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(info.height).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(info.start_time).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(info.fail_time).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(info.speed).to_vec());

        Ok(())
    }

    pub(crate) fn append_frame(vec: &mut Vec<u8>, frame: &Frame) {
        vec.append(&mut ReplayFloat::to_le_bytes(frame.time).to_vec());
        vec.append(&mut ReplayInt::to_le_bytes(frame.fps).to_vec());
        append_position_and_rotation(vec, &frame.head);
        append_position_and_rotation(vec, &frame.left_hand);
        append_position_and_rotation(vec, &frame.right_hand);
    }

    pub(crate) fn append_note_cut_info(vec: &mut Vec<u8>, cut_info: &NoteCutInfo) {
        vec.push(cut_info.speed_ok as u8);
        vec.push(cut_info.direction_ok as u8);
        vec.push(cut_info.saber_type_ok as u8);
        vec.push(cut_info.was_cut_too_soon as u8);
        vec.append(&mut ReplayFloat::to_le_bytes(cut_info.saber_speed).to_vec());
        append_vector3(vec, &cut_info.saber_dir);

        let saber_type: u8 = cut_info.saber_type.try_into().unwrap();
        vec.append(&mut ReplayInt::to_le_bytes(saber_type as ReplayInt).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(cut_info.time_deviation).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(cut_info.cut_dir_deviation).to_vec());
        append_vector3(vec, &cut_info.cut_point);
        append_vector3(vec, &cut_info.cut_normal);
        vec.append(&mut ReplayFloat::to_le_bytes(cut_info.cut_distance_to_center).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(cut_info.cut_angle).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(cut_info.before_cut_rating).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(cut_info.after_cut_rating).to_vec());
    }

    pub(crate) fn append_note(vec: &mut Vec<u8>, note: &Note) {
        let scoring_type_u8: u8 = NoteScoringType::try_into(note.scoring_type).unwrap();
        let color_type_u8: u8 = ColorType::try_into(note.color_type).unwrap();
        let cut_direction_u8: u8 = CutDirection::try_into(note.cut_direction).unwrap();

        let note_id: ReplayInt = scoring_type_u8 as ReplayInt * 10000
            + note.line_idx as ReplayInt * 1000
            + note.line_layer as ReplayInt * 100
            + color_type_u8 as ReplayInt * 10
            + cut_direction_u8 as ReplayInt;
        vec.append(&mut ReplayInt::to_le_bytes(note_id).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(note.event_time).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(note.spawn_time).to_vec());

        let event_type: u8 = note.event_type.try_into().unwrap();
        vec.append(&mut ReplayInt::to_le_bytes(event_type as ReplayInt).to_vec());

        match note.event_type {
            NoteEventType::Good | NoteEventType::Bad => {
                append_note_cut_info(vec, note.cut_info.as_ref().unwrap())
            }
            _ => {}
        }
    }

    pub(crate) fn append_wall(vec: &mut Vec<u8>, wall: &Wall) {
        let wall_id: ReplayInt = wall.line_idx as ReplayInt * 100
            + wall.obstacle_type as ReplayInt * 10
            + wall.width as ReplayInt;
        vec.append(&mut ReplayInt::to_le_bytes(wall_id).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(wall.energy).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(wall.time).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(wall.spawn_time).to_vec());
    }

    pub(crate) fn append_height(vec: &mut Vec<u8>, height: &Height) {
        vec.append(&mut ReplayFloat::to_le_bytes(height.height).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(height.time).to_vec());
    }

    pub(crate) fn append_pause(vec: &mut Vec<u8>, pause: &Pause) {
        vec.append(&mut ReplayLong::to_le_bytes(pause.duration).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(pause.time).to_vec());
    }

    pub(crate) fn generate_random_position_and_rotation() -> PositionAndRotation {
        PositionAndRotation {
            position: generate_random_vec3(),
            rotation: generate_random_vec4(),
        }
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

    pub(crate) fn generate_random_replay() -> Replay {
        Replay {
            version: 1,
            info: generate_random_info(),
            frames: Frames::new(Vec::from([
                generate_random_frame(),
                generate_random_frame(),
            ])),
            notes: Notes::new(Vec::from([
                generate_random_note(NoteEventType::Bomb),
                generate_random_note(NoteEventType::Good),
            ])),
            walls: Walls::new(Vec::from([generate_random_wall(), generate_random_wall()])),
            heights: Heights::new(Vec::from([
                generate_random_height(),
                generate_random_height(),
            ])),
            pauses: Pauses::new(Vec::from([
                generate_random_pause(),
                generate_random_pause(),
            ])),
        }
    }

    pub(crate) fn generate_random_info() -> Info {
        let version = "0.5.4".to_owned();
        let game_version = "1.27.0".to_owned();
        let timestamp = random::<u32>().to_string();
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
        let score = (random::<u32>() % 2_000_000) as i32;
        let mode = "Standard".to_owned();
        let environment = "Timbaland".to_owned();
        let modifiers = "DA,FS".to_owned();
        let jump_distance = random::<ReplayFloat>() * 25.0;
        let left_handed = false;
        let height = random::<ReplayFloat>() * 2.0;
        let start_time = 0.0f32;
        let fail_time = 0.0f32;
        let speed = 0.0f32;

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
    }

    pub(crate) fn generate_random_note_cut_info() -> NoteCutInfo {
        NoteCutInfo {
            speed_ok: random::<bool>(),
            direction_ok: random::<bool>(),
            saber_type_ok: random::<bool>(),
            was_cut_too_soon: random::<bool>(),
            saber_speed: random::<ReplayFloat>(),
            saber_dir: generate_random_vec3(),
            saber_type: ColorType::try_from(random::<u8>() % 2).unwrap(),
            time_deviation: random::<ReplayFloat>(),
            cut_dir_deviation: random::<ReplayFloat>(),
            cut_point: generate_random_vec3(),
            cut_normal: generate_random_vec3(),
            cut_distance_to_center: random::<ReplayFloat>(),
            cut_angle: random::<ReplayFloat>(),
            before_cut_rating: random::<ReplayFloat>(),
            after_cut_rating: random::<ReplayFloat>(),
        }
    }

    pub(crate) fn generate_random_note(event_type: NoteEventType) -> Note {
        let cut_info = match &event_type {
            _x @ NoteEventType::Good | _x @ NoteEventType::Bad => {
                Some(generate_random_note_cut_info())
            }
            _ => None,
        };

        Note {
            scoring_type: NoteScoringType::Normal,
            line_idx: random::<u8>() % 4,
            line_layer: random::<u8>() % 3,
            color_type: ColorType::try_from(random::<u8>() % 2).unwrap(),
            cut_direction: CutDirection::try_from(random::<u8>() % 9).unwrap(),
            event_time: random::<ReplayTime>() * 100.0,
            spawn_time: random::<ReplayTime>() * 100.0,
            event_type,
            cut_info,
        }
    }

    pub(crate) fn generate_random_frame() -> Frame {
        Frame {
            time: random::<ReplayFloat>() * 100.0,
            fps: random::<ReplayInt>() % 144,
            head: generate_random_position_and_rotation(),
            left_hand: generate_random_position_and_rotation(),
            right_hand: generate_random_position_and_rotation(),
        }
    }

    pub(crate) fn generate_random_wall() -> Wall {
        Wall {
            line_idx: random::<u8>() % 4,
            obstacle_type: random::<u8>() % 10,
            width: random::<u8>() % 4,
            energy: random::<ReplayFloat>() * 100.0,
            time: random::<ReplayFloat>() * 100.0,
            spawn_time: random::<ReplayFloat>() * 100.0,
        }
    }

    pub(crate) fn generate_random_height() -> Height {
        Height {
            height: random::<ReplayFloat>() * 2.0,
            time: random::<ReplayFloat>() * 100.0,
        }
    }

    pub(crate) fn generate_random_pause() -> Pause {
        Pause {
            duration: random::<ReplayLong>() % 30,
            time: random::<ReplayFloat>() * 100.0,
        }
    }

    pub(crate) fn get_replay_buffer(replay: &Replay) -> Result<Vec<u8>> {
        // header
        let mut buf = ReplayInt::to_le_bytes(BSOR_MAGIC).to_vec();
        buf.push(replay.version);

        // info
        let info_id = BlockType::Info.try_into()?;
        buf.append(&mut Vec::from([info_id]));
        append_info(&mut buf, &replay.info)?;

        buf.append(&mut get_frames_buffer(replay.frames.get_vec())?);
        buf.append(&mut get_notes_buffer(replay.notes.get_vec())?);
        buf.append(&mut get_walls_buffer(replay.walls.get_vec())?);
        buf.append(&mut get_heights_buffer(replay.heights.get_vec())?);
        buf.append(&mut get_pauses_buffer(replay.pauses.get_vec())?);

        Ok(buf)
    }

    pub(crate) fn get_frames_buffer(frames: &Vec<Frame>) -> Result<Vec<u8>> {
        let frames_id = BlockType::Frames.try_into()?;
        let mut buf: Vec<u8> = Vec::from([frames_id]);

        buf.append(&mut ReplayInt::to_le_bytes(frames.len() as ReplayInt).to_vec());
        for f in frames.iter() {
            append_frame(&mut buf, &f);
        }

        Ok(buf)
    }

    pub(crate) fn get_notes_buffer(notes: &Vec<Note>) -> Result<Vec<u8>> {
        let notes_id = BlockType::Notes.try_into()?;
        let mut buf: Vec<u8> = Vec::from([notes_id]);

        buf.append(&mut ReplayInt::to_le_bytes(notes.len() as ReplayInt).to_vec());
        for f in notes.iter() {
            append_note(&mut buf, &f);
        }

        Ok(buf)
    }

    pub(crate) fn get_walls_buffer(walls: &Vec<Wall>) -> Result<Vec<u8>> {
        let walls_id = BlockType::Walls.try_into()?;
        let mut buf: Vec<u8> = Vec::from([walls_id]);

        buf.append(&mut ReplayInt::to_le_bytes(walls.len() as ReplayInt).to_vec());
        for f in walls.iter() {
            append_wall(&mut buf, &f);
        }

        Ok(buf)
    }

    pub(crate) fn get_heights_buffer(heights: &Vec<Height>) -> Result<Vec<u8>> {
        let heights_id = BlockType::Heights.try_into()?;
        let mut buf: Vec<u8> = Vec::from([heights_id]);

        buf.append(&mut ReplayInt::to_le_bytes(heights.len() as ReplayInt).to_vec());
        for f in heights.iter() {
            append_height(&mut buf, &f);
        }

        Ok(buf)
    }

    pub(crate) fn get_pauses_buffer(pauses: &Vec<Pause>) -> Result<Vec<u8>> {
        let pauses_id = BlockType::Pauses.try_into()?;
        let mut buf: Vec<u8> = Vec::from([pauses_id]);

        buf.append(&mut ReplayInt::to_le_bytes(pauses.len() as ReplayInt).to_vec());
        for f in pauses.iter() {
            append_pause(&mut buf, &f);
        }

        Ok(buf)
    }
}
