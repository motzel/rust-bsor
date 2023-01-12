use crate::replay::{
    read_utils, vector::Vector3, BsorError, LineValue, ReplayFloat, ReplayInt, ReplayTime, Result,
};
use std::io::Read;

#[derive(Debug)]
pub struct Notes(Vec<Note>);

impl Notes {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Notes> {
        match read_utils::read_byte(r) {
            Ok(v) => {
                if v != 2 {
                    return Err(BsorError::InvalidBsor);
                }
            }
            Err(e) => return Err(e),
        }

        let count = read_utils::read_int(r)? as usize;
        let mut vec = Vec::<Note>::with_capacity(count);

        for _ in 0..count {
            vec.push(Note::load(r)?);
        }

        Ok(Notes(vec))
    }

    pub fn get_vec(&self) -> &Vec<Note> {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
}

type LayerValue = u8;

#[derive(Debug, PartialEq)]
pub struct Note {
    pub scoring_type: NoteScoringType,
    pub line_idx: LineValue,
    pub line_layer: LayerValue,
    pub color_type: ColorType,
    pub cut_direction: CutDirection,
    pub event_time: ReplayTime,
    pub spawn_time: ReplayTime,
    pub event_type: NoteEventType,
    pub cut_info: Option<NoteCutInfo>,
}

impl Note {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Note> {
        let mut note_id = read_utils::read_int(r)?;

        let scoring_type = NoteScoringType::try_from((note_id / 10000) as u8)?;
        note_id %= 10000;

        let line_idx = (note_id / 1000) as LineValue;
        note_id %= 1000;

        let line_layer = (note_id / 100) as LayerValue;
        note_id %= 100;

        let color_type = ColorType::try_from((note_id / 10) as u8)?;
        note_id %= 10;

        let cut_direction = CutDirection::try_from(note_id as u8)?;

        let event_time = read_utils::read_float(r)?;
        let spawn_time = read_utils::read_float(r)?;
        let event_type = NoteEventType::try_from(read_utils::read_int(r)?)?;

        let cut_info = match &event_type {
            _x @ NoteEventType::Good | _x @ NoteEventType::Bad => Some(NoteCutInfo::load(r)?),
            _ => None,
        };

        Ok(Note {
            scoring_type,
            line_idx,
            line_layer,
            color_type,
            cut_direction,
            event_time,
            spawn_time,
            event_type,
            cut_info,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct NoteCutInfo {
    pub speed_ok: bool,
    pub direction_ok: bool,
    pub saber_type_ok: bool,
    pub was_cut_too_soon: bool,
    pub saber_speed: ReplayFloat,
    pub saber_dir: Vector3,
    pub saber_type: ColorType,
    pub time_deviation: ReplayFloat,
    pub cut_dir_deviation: ReplayFloat,
    pub cut_point: Vector3,
    pub cut_normal: Vector3,
    pub cut_distance_to_center: ReplayFloat,
    pub cut_angle: ReplayFloat,
    pub before_cut_rating: ReplayFloat,
    pub after_cut_rating: ReplayFloat,
}

impl NoteCutInfo {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<NoteCutInfo> {
        let speed_ok = read_utils::read_bool(r)?;
        let direction_ok = read_utils::read_bool(r)?;
        let saber_type_ok = read_utils::read_bool(r)?;
        let was_cut_too_soon = read_utils::read_bool(r)?;
        let saber_speed = read_utils::read_float(r)?;
        let saber_dir = Vector3::load(r)?;
        let saber_type = ColorType::try_from(read_utils::read_int(r)? as u8)?;
        let time_deviation = read_utils::read_float(r)?;
        let cut_dir_deviation = read_utils::read_float(r)?;
        let cut_point = Vector3::load(r)?;
        let cut_normal = Vector3::load(r)?;
        let cut_distance_to_center = read_utils::read_float(r)?;
        let cut_angle = read_utils::read_float(r)?;
        let before_cut_rating = read_utils::read_float(r)?;
        let after_cut_rating = read_utils::read_float(r)?;

        Ok(NoteCutInfo {
            speed_ok,
            direction_ok,
            saber_type_ok,
            was_cut_too_soon,
            saber_speed,
            saber_dir,
            saber_type,
            time_deviation,
            cut_dir_deviation,
            cut_point,
            cut_normal,
            cut_distance_to_center,
            cut_angle,
            before_cut_rating,
            after_cut_rating,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NoteEventType {
    Good = 0,
    Bad,
    Miss,
    Bomb,
    Unknown = 255,
}

impl TryFrom<ReplayInt> for NoteEventType {
    type Error = BsorError;

    fn try_from(v: ReplayInt) -> std::result::Result<Self, Self::Error> {
        match v {
            x if x == NoteEventType::Good as ReplayInt => Ok(NoteEventType::Good),
            x if x == NoteEventType::Bad as ReplayInt => Ok(NoteEventType::Bad),
            x if x == NoteEventType::Miss as ReplayInt => Ok(NoteEventType::Miss),
            x if x == NoteEventType::Bomb as ReplayInt => Ok(NoteEventType::Bomb),
            _ => Ok(NoteEventType::Unknown),
        }
    }
}

impl TryInto<u8> for NoteEventType {
    type Error = BsorError;

    fn try_into(self) -> std::result::Result<u8, Self::Error> {
        Ok(self as u8)
    }
}

impl PartialEq for NoteEventType {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NoteScoringType {
    NormalOld = 0,
    Ignore,
    NoScore,
    Normal,
    SliderHead,
    SliderTail,
    BurstSliderHead,
    BurstSliderElement,
    Unknown = 255,
}

impl TryFrom<u8> for NoteScoringType {
    type Error = BsorError;

    fn try_from(v: u8) -> std::result::Result<Self, Self::Error> {
        match v {
            x if x == NoteScoringType::NormalOld as u8 => Ok(NoteScoringType::NormalOld),
            x if x == NoteScoringType::Ignore as u8 => Ok(NoteScoringType::Ignore),
            x if x == NoteScoringType::NoScore as u8 => Ok(NoteScoringType::NoScore),
            x if x == NoteScoringType::Normal as u8 => Ok(NoteScoringType::Normal),
            x if x == NoteScoringType::SliderHead as u8 => Ok(NoteScoringType::SliderHead),
            x if x == NoteScoringType::SliderTail as u8 => Ok(NoteScoringType::SliderTail),
            x if x == NoteScoringType::BurstSliderHead as u8 => {
                Ok(NoteScoringType::BurstSliderHead)
            }
            x if x == NoteScoringType::BurstSliderElement as u8 => {
                Ok(NoteScoringType::BurstSliderElement)
            }
            _ => Ok(NoteScoringType::Unknown),
        }
    }
}

impl TryInto<u8> for NoteScoringType {
    type Error = BsorError;

    fn try_into(self) -> std::result::Result<u8, Self::Error> {
        Ok(self as u8)
    }
}

impl PartialEq for NoteScoringType {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CutDirection {
    TopCenter,
    BottomCenter,
    MiddleLeft,
    MiddleRight,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Dot,
    Unknown = 255,
}

impl TryFrom<u8> for CutDirection {
    type Error = BsorError;

    fn try_from(v: u8) -> std::result::Result<Self, Self::Error> {
        match v {
            x if x == CutDirection::TopCenter as u8 => Ok(CutDirection::TopCenter),
            x if x == CutDirection::BottomCenter as u8 => Ok(CutDirection::BottomCenter),
            x if x == CutDirection::MiddleLeft as u8 => Ok(CutDirection::MiddleLeft),
            x if x == CutDirection::MiddleRight as u8 => Ok(CutDirection::MiddleRight),
            x if x == CutDirection::TopLeft as u8 => Ok(CutDirection::TopLeft),
            x if x == CutDirection::TopRight as u8 => Ok(CutDirection::TopRight),
            x if x == CutDirection::BottomLeft as u8 => Ok(CutDirection::BottomLeft),
            x if x == CutDirection::BottomRight as u8 => Ok(CutDirection::BottomRight),
            x if x == CutDirection::Dot as u8 => Ok(CutDirection::Dot),
            _ => Ok(CutDirection::Unknown),
        }
    }
}

impl TryInto<u8> for CutDirection {
    type Error = BsorError;

    fn try_into(self) -> std::result::Result<u8, Self::Error> {
        Ok(self as u8)
    }
}

impl PartialEq for CutDirection {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ColorType {
    Red = 0,
    Blue,
    Unknown = 255,
}

impl TryFrom<u8> for ColorType {
    type Error = BsorError;

    fn try_from(v: u8) -> std::result::Result<Self, Self::Error> {
        match v {
            x if x == ColorType::Red as u8 => Ok(ColorType::Red),
            x if x == ColorType::Blue as u8 => Ok(ColorType::Blue),
            _ => Ok(ColorType::Unknown),
        }
    }
}

impl TryInto<u8> for ColorType {
    type Error = BsorError;

    fn try_into(self) -> std::result::Result<u8, Self::Error> {
        Ok(self as u8)
    }
}

impl PartialEq for ColorType {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_util::{append_vector3, generate_random_vec3};
    use rand::random;
    use std::io::Cursor;

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

    fn append_note_cut_info(vec: &mut Vec<u8>, cut_info: &NoteCutInfo) {
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

    fn append_note(vec: &mut Vec<u8>, note: &Note) {
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

    #[test]
    fn it_can_load_good_note() {
        let note = generate_random_note(NoteEventType::Good);

        let mut buf: Vec<u8> = Vec::new();
        append_note(&mut buf, &note);

        let result = Note::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(result, note)
    }

    #[test]
    fn it_can_load_bad_note() {
        let note = generate_random_note(NoteEventType::Bad);

        let mut buf: Vec<u8> = Vec::new();
        append_note(&mut buf, &note);

        let result = Note::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(result, note)
    }

    #[test]
    fn it_can_load_note_without_note_cut_info() {
        let note = generate_random_note(NoteEventType::Miss);

        let mut buf: Vec<u8> = Vec::new();
        append_note(&mut buf, &note);

        let result = Note::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(result, note)
    }

    #[test]
    fn it_can_load_notes() {
        let notess = Vec::from([
            generate_random_note(NoteEventType::Bomb),
            generate_random_note(NoteEventType::Good),
        ]);

        let mut buf: Vec<u8> = Vec::from([2u8]);
        buf.append(&mut ReplayInt::to_le_bytes(notess.len() as ReplayInt).to_vec());
        for n in notess.iter() {
            append_note(&mut buf, &n);
        }

        let result = Notes::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(*result.get_vec(), notess)
    }
}
