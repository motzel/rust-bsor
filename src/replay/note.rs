use crate::replay::{
    assert_start_of_block, read_utils, vector::Vector3, BlockType, BsorError, GetStaticBlockSize,
    LineValue, LoadBlock, LoadRealBlockSize, ParsedReplayBlock, ReplayFloat, ReplayInt, ReplayTime,
    Result,
};
use std::io::{Read, Seek, SeekFrom};
use std::marker::PhantomData;
use std::mem::size_of;

#[derive(Debug)]
pub struct Notes(Vec<Note>);

impl Notes {
    #[cfg(test)]
    pub(crate) fn new(vec: Vec<Note>) -> Notes {
        Notes(vec)
    }

    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Notes> {
        assert_start_of_block(r, BlockType::Notes)?;

        let count = read_utils::read_int(r)? as usize;
        let mut vec = Vec::<Note>::with_capacity(count);

        for _ in 0..count {
            vec.push(Note::load(r)?);
        }

        Ok(Notes(vec))
    }

    pub fn load_block<RS: Read + Seek>(
        r: &mut RS,
        block: &ParsedReplayBlock<Notes>,
    ) -> Result<Self> {
        r.seek(SeekFrom::Start(block.pos))?;

        Self::load(r)
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

impl GetStaticBlockSize for Notes {
    fn get_static_size() -> usize {
        size_of::<u8>() + size_of::<ReplayInt>()
    }
}

impl LoadBlock for ParsedReplayBlock<Notes> {
    type Item = Notes;

    fn load<RS: Read + Seek>(&self, r: &mut RS) -> Result<Self::Item> {
        Self::Item::load_block(r, self)
    }
}

impl LoadRealBlockSize for Notes {
    type Item = Notes;

    fn load_real_block_size<RS: Read + Seek>(
        r: &mut RS,
        pos: u64,
    ) -> Result<ParsedReplayBlock<Notes>> {
        assert_start_of_block(r, BlockType::Notes)?;

        let count = read_utils::read_int(r)?;

        let mut bytes = Notes::get_static_size() as u64;
        let mut current_pos = pos + bytes;
        for _ in 0..count {
            let note_bytes = Note::get_total_block_size(r)?;
            bytes += note_bytes;

            current_pos += note_bytes;
            r.seek(SeekFrom::Start(current_pos))?;
        }

        Ok(ParsedReplayBlock::<Notes> {
            pos,
            bytes,
            items_count: count,
            _phantom: PhantomData,
        })
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

    pub(self) fn get_total_block_size<RS: Read + Seek>(r: &mut RS) -> Result<u64> {
        // skip to event type field
        r.seek(SeekFrom::Current(
            size_of::<ReplayInt>() as i64 + size_of::<ReplayFloat>() as i64 * 2,
        ))?;

        let event_type = NoteEventType::try_from(read_utils::read_int(r)?)?;

        let bytes = Note::get_static_size() as u64
            + match &event_type {
                _x @ NoteEventType::Good | _x @ NoteEventType::Bad => {
                    NoteCutInfo::get_static_size() as u64
                }
                _ => 0,
            };

        Ok(bytes)
    }
}

impl GetStaticBlockSize for Note {
    fn get_static_size() -> usize {
        size_of::<ReplayInt>() * 2 + size_of::<ReplayFloat>() * 2
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
impl GetStaticBlockSize for NoteCutInfo {
    fn get_static_size() -> usize {
        size_of::<u8>() * 4
            + size_of::<ReplayInt>()
            + size_of::<ReplayFloat>() * 7
            + size_of::<Vector3>() * 3
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
    use crate::tests_util::{append_note, generate_random_note, get_notes_buffer};
    use std::io::Cursor;

    #[test]
    fn it_returns_correct_static_size_of_note() {
        assert_eq!(Note::get_static_size(), 16);
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
    fn it_returns_correct_static_size_of_notes() {
        assert_eq!(Notes::get_static_size(), 5);
    }

    #[test]
    fn it_returns_invalid_bsor_error_when_notes_block_id_is_invalid() -> Result<()> {
        let notes = Vec::from([
            generate_random_note(NoteEventType::Bomb),
            generate_random_note(NoteEventType::Good),
        ]);

        let mut buf = get_notes_buffer(&notes)?;
        buf[0] = 255;

        let result = Notes::load(&mut Cursor::new(buf));

        assert!(matches!(result, Err(BsorError::InvalidBsor)));

        Ok(())
    }

    #[test]
    fn it_can_load_notes() -> Result<()> {
        let notes = Vec::from([
            generate_random_note(NoteEventType::Bomb),
            generate_random_note(NoteEventType::Good),
        ]);

        let buf = get_notes_buffer(&notes)?;

        let result = Notes::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(*result.get_vec(), notes);
        assert_eq!(result.is_empty(), false);
        assert_eq!(result.len(), notes.len());

        Ok(())
    }

    #[test]
    fn it_can_load_parsed_notes_block() -> Result<()> {
        let notes = Vec::from([
            generate_random_note(NoteEventType::Good),
            generate_random_note(NoteEventType::Bad),
            generate_random_note(NoteEventType::Miss),
            generate_random_note(NoteEventType::Miss),
            generate_random_note(NoteEventType::Bomb),
        ]);

        let buf = get_notes_buffer(&notes)?;

        let pos = 0;
        let reader = &mut Cursor::new(buf);
        let notes_block = Notes::load_real_block_size(reader, pos)?;

        let result = notes_block.load(reader)?;

        assert_eq!(notes_block.pos(), pos);
        assert_eq!(
            notes_block.bytes(),
            Notes::get_static_size() as u64 + 88 * 2 + 16 * 3
        );
        assert_eq!(notes_block.is_empty(), false);
        assert_eq!(notes_block.len(), notes.len() as i32);
        assert_eq!(*result.get_vec(), notes);

        Ok(())
    }
}
