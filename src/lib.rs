use pyo3::basic::CompareOp;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::{create_exception, wrap_pyfunction};

use imageparse::chd::ChdImageError;
use imageparse::{Image, MsfIndexError};

//library base exception, for python to be able to 'catch the rest' or 'catch everything'
create_exception!(chdimage, ImageError, PyException);
create_exception!(chdimage, HdChdError, ImageError);
create_exception!(chdimage, GdiChdError, ImageError);
create_exception!(chdimage, OrphanError, ImageError);

#[rustfmt::skip]
fn from_chdlibrary(err: ChdImageError) -> PyErr {
    match err {
        ChdImageError::ChdError(e)                => ImageError::new_err(format!("{}", e)),
        ChdImageError::IoError(e)                 => ImageError::new_err(format!("{}", e)),
        ChdImageError::TrackParseError(e)         => ImageError::new_err(format!("{}", e)),
        ChdImageError::WrongHunkSize              => HdChdError::new_err("Hard disc chd are not supported"),
        ChdImageError::WrongBufferSize            => unreachable!(), //the buffer is created with the chd struct, not passed from user
        ChdImageError::UnsupportedSectorFormat(e) => ImageError::new_err(e),
        ChdImageError::HunkRecvError(_)           => unreachable!(), //multithreading is turned off in the features
        ChdImageError::NoTracks                   => GdiChdError::new_err("Gdi chd are not supported"),
        ChdImageError::RecursionDepthExceeded     => ImageError::new_err("Chd parent recursion depth exceeded"),
        ChdImageError::UnsupportedChdVersion      => ImageError::new_err("Chd parent is only supported for chd V3, V4 and V5"),
        ChdImageError::ParentNotFound             => OrphanError::new_err("Chd parent was not in the path arguments"),
    }
}
#[rustfmt::skip]
fn from_imagelibrary(err: imageparse::ImageError) -> PyErr {
    match err {
        imageparse::ImageError::UnsupportedFormat => ImageError::new_err("Unsupported image format"),
        imageparse::ImageError::CueError(e)       => ImageError::new_err(format!("{}", e)),
        imageparse::ImageError::ChdError(e)       => ImageError::new_err(format!("{}", e)),
        imageparse::ImageError::MsfIndexError(e)  => ImageError::new_err(format!("{}", e)),
        imageparse::ImageError::IoError(e)        => ImageError::new_err(format!("{}", e)),
        imageparse::ImageError::OutOfRange        => ImageError::new_err("Out of Range"),
    }
}
#[rustfmt::skip]
fn from_msflibrary(err: MsfIndexError) -> PyErr {
    match err {
        MsfIndexError::ParseIntError(e) => ImageError::new_err(e),
        MsfIndexError::OutOfRangeError  => ImageError::new_err("Out of Range MSF"),
        MsfIndexError::InvalidMsfError  => ImageError::new_err("Invalid MSF"),
    }
}

#[pyclass]
struct ChdImage {
    inner: imageparse::chd::ChdImage,
    buf: [u8; 2352],
}

#[derive(PartialEq, Eq)]
#[pyclass]
struct TrackType(usize);

impl TrackType {
    fn new(track: imageparse::TrackType) -> TrackType {
        match track {
            imageparse::TrackType::Mode1 => TrackType(1),
            imageparse::TrackType::Mode2 => TrackType(2),
            imageparse::TrackType::Audio => TrackType(3),
        }
    }
}

#[allow(non_snake_case)]
#[pymethods]
impl TrackType {
    #[classattr]
    fn MODE1() -> TrackType {
        TrackType(1)
    }
    #[classattr]
    fn MODE2() -> TrackType {
        TrackType(2)
    }
    #[classattr]
    fn AUDIO() -> TrackType {
        TrackType(3)
    }
    fn __str__(&self) -> String {
        match self {
            TrackType(1) => "MODE1_RAW".to_string(),
            TrackType(2) => "MODE2_RAW".to_string(),
            TrackType(3) => "AUDIO".to_string(),
            _ => unreachable!(),
        }
    }
    fn __repr__(&self) -> String {
        self.__str__()
    }
    fn __richcmp__(&self, other: PyRef<Event>, op: CompareOp) -> Py<PyAny> {
        let py = other.py();
        match op {
            CompareOp::Eq => (self.0 == other.0).into_py(py),
            CompareOp::Ne => (self.0 != other.0).into_py(py),
            _ => py.NotImplemented(),
        }
    }
}

#[derive(PartialEq, Eq)]
#[pyclass]
struct Event(usize);

impl Event {
    fn new(track: imageparse::Event) -> Event {
        match track {
            imageparse::Event::TrackChange => Event(1),
            imageparse::Event::EndOfDisc => Event(2),
        }
    }
}

#[allow(non_snake_case)]
#[pymethods]
impl Event {
    #[classattr]
    fn TRACKCHANGE() -> Event {
        Event(1)
    }
    #[classattr]
    fn ENDOFDISC() -> Event {
        Event(2)
    }
    fn __str__(&self) -> String {
        match self {
            Event(1) => "TrackChange".to_string(),
            Event(2) => "EndOfDisc".to_string(),
            _ => unreachable!(),
        }
    }
    fn __repr__(&self) -> String {
        self.__str__()
    }
    fn __richcmp__(&self, other: PyRef<Event>, op: CompareOp) -> Py<PyAny> {
        let py = other.py();
        match op {
            CompareOp::Eq => (self.0 == other.0).into_py(py),
            CompareOp::Ne => (self.0 != other.0).into_py(py),
            _ => py.NotImplemented(),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
#[pyclass]
pub struct MsfIndex(imageparse::MsfIndex);

#[pymethods]
impl MsfIndex {
    #[new]
    pub fn new(m: u8, s: u8, f: u8) -> PyResult<MsfIndex> {
        Ok(MsfIndex(
            imageparse::MsfIndex::new(m, s, f).map_err(from_msflibrary)?,
        ))
    }
    #[staticmethod]
    pub fn from_bcd_values(m_bcd: u8, s_bcd: u8, f_bcd: u8) -> PyResult<MsfIndex> {
        Ok(MsfIndex(
            imageparse::MsfIndex::from_bcd_values(m_bcd, s_bcd, f_bcd).map_err(from_msflibrary)?,
        ))
    }
    #[staticmethod]
    pub fn try_from_str(s: &str) -> PyResult<MsfIndex> {
        Ok(MsfIndex(
            imageparse::MsfIndex::try_from_str(s).map_err(from_msflibrary)?,
        ))
    }
    #[staticmethod]
    pub fn from_lba(sector_no: u32) -> PyResult<MsfIndex> {
        Ok(MsfIndex(
            imageparse::MsfIndex::from_lba(sector_no).map_err(from_msflibrary)?,
        ))
    }
    pub fn to_lba(&self) -> u32 {
        self.0.to_lba()
    }
    pub fn to_bcd_values(&self) -> (u8, u8, u8) {
        self.0.to_bcd_values()
    }
    fn __str__(&self) -> String {
        format!("MsfIndex{}", self.0)
    }
    fn __repr__(&self) -> String {
        format!("MsfIndex{}", self.0)
    }
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Lt => self.0 < other.0,
            CompareOp::Le => self.0 <= other.0,
            CompareOp::Eq => self.0 == other.0,
            CompareOp::Ne => self.0 != other.0,
            CompareOp::Gt => self.0 > other.0,
            CompareOp::Ge => self.0 >= other.0,
        }
    }
}

#[pymethods]
impl ChdImage {
    fn num_tracks(&self) -> usize {
        self.inner.num_tracks()
    }
    fn current_subchannel_q_valid(&self) -> bool {
        self.inner.current_subchannel_q_valid()
    }
    fn current_track(&self) -> PyResult<u8> {
        self.inner.current_track().map_err(from_imagelibrary)
    }
    fn current_index(&self) -> PyResult<u8> {
        self.inner.current_index().map_err(from_imagelibrary)
    }
    fn current_track_type(&self) -> PyResult<TrackType> {
        let track = self.inner.current_track_type().map_err(from_imagelibrary)?;
        Ok(TrackType::new(track))
    }
    fn first_track_type(&self) -> TrackType {
        TrackType::new(self.inner.first_track_type())
    }
    fn track_sha1s(&mut self) -> PyResult<Vec<[u8; 20]>> {
        self.inner.track_sha1s().map_err(from_imagelibrary)
    }
    fn set_location_to_track(&mut self, track: u8) -> PyResult<()> {
        self.inner
            .set_location_to_track(track)
            .map_err(from_imagelibrary)
    }
    fn advance_position(&mut self) -> PyResult<Option<Event>> {
        let event = self.inner.advance_position().map_err(from_imagelibrary)?;
        Ok(event.map(Event::new))
    }
    /// `buf` is expected to be 2352 bytes long
    fn copy_current_sector(&mut self) -> PyResult<&[u8]> {
        self.inner
            .copy_current_sector(&mut self.buf)
            .map_err(from_imagelibrary)?;
        Ok(&self.buf)
    }
    fn current_track_local_msf(&self) -> PyResult<MsfIndex> {
        Ok(MsfIndex(
            self.inner
                .current_track_local_msf()
                .map_err(from_imagelibrary)?,
        ))
    }
    fn current_global_msf(&self) -> PyResult<MsfIndex> {
        Ok(MsfIndex(
            self.inner.current_global_msf().map_err(from_imagelibrary)?,
        ))
    }
    fn track_start(&self, track: u8) -> PyResult<MsfIndex> {
        Ok(MsfIndex(
            self.inner.track_start(track).map_err(from_imagelibrary)?,
        ))
    }
    fn set_location(&mut self, target: MsfIndex) -> PyResult<()> {
        self.inner
            .set_location(target.0)
            .map_err(from_imagelibrary)?;
        Ok(())
    }
}

#[pyfunction]
fn open_with_parent(path: String, possible_parents: Vec<String>) -> PyResult<ChdImage> {
    Ok(ChdImage {
        inner: imageparse::chd::ChdImage::open_with_parent(path, &possible_parents)
            .map_err(from_chdlibrary)?,
        buf: [0u8; 2352],
    })
}

#[pyfunction]
fn open(path: String) -> PyResult<ChdImage> {
    Ok(ChdImage {
        inner: imageparse::chd::ChdImage::open(path).map_err(from_chdlibrary)?,
        buf: [0u8; 2352],
    })
}

#[pymodule]
fn chdimage(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(open_with_parent, m)?)?;
    m.add_function(wrap_pyfunction!(open, m)?)?;
    m.add("ImageError", _py.get_type::<ImageError>())?;
    m.add("HdChdError", _py.get_type::<HdChdError>())?;
    m.add("GdiChdError", _py.get_type::<GdiChdError>())?;
    m.add("OrphanError", _py.get_type::<OrphanError>())?;
    m.add("MsfIndex", _py.get_type::<MsfIndex>())?;
    m.add("Event", _py.get_type::<Event>())?;
    m.add("TrackType", _py.get_type::<TrackType>())?;
    Ok(())
}
