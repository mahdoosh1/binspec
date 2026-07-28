use binspec::{
    ByteSource, View, array, assert_spec, assert_spec_eq, default_main, errors::SResult, spec_error, specs::{
        Spec, U8, try_spec
    }
};

fn parse_int(bytes: &[u8], radix: u32) -> Option<u64> {
    let s = std::str::from_utf8(bytes).ok()?;
    u64::from_str_radix(s.trim_end_matches(&[' ', '\0'][..]), radix).ok()
}

// ----------------------------------------------------------------------
// TarString

#[derive(Debug)]
pub struct TarString {
    pub text: Option<Vec<u8>>
}

impl Spec for TarString {
    type Params = usize;                     // size of the whole field (string + terminator)
    fn read_all<S: ByteSource>(data: &S, size: Self::Params) -> SResult<(Self, usize)> {
        let mut view = View::from(data); // at 0
        // string part
        let string_bytes = view.consume_n(size - 1)?; // at size - 1
        // terminator
        let end = U8::read_from_view(&mut view, ())?; // at size
        assert_spec!(end.value == 0 || end.value == b' ', "Tar string terminator invalid");

        // for i in string { if i != 0 && i != b' ' { break string } }
        // -> Some(string) if any byte is neither NUL nor space, otherwise None
        let text = if string_bytes.iter().any(|&b| b != 0 && b != b' ') {
            Some(string_bytes.into())
        } else {
            None
        };

        Ok((TarString { text }, view.cursor.offset)) // view.cursor.offset = size
    }
}

// ----------------------------------------------------------------------
// Oct – octal numeric field

#[derive(Debug)]
pub struct Oct {
    pub value: Option<u64>
}

impl Spec for Oct {
    type Params = Option<usize>;    // defaults to 8 when not specified
    fn read_all<S: ByteSource>(data: &S, size: Self::Params) -> SResult<(Self, usize)> {
        let mut view = View::from(data); // at 0
        let size = size.unwrap_or(8);

        let digits = TarString::read_from_view(&mut view, size)?.text; // at size (calculated from TarString)
        let value = digits.and_then(|digits| parse_int(digits.as_ref(), 8));

        Ok((Oct { value }, view.cursor.offset)) // view.cursor.offset = size
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum Header_FileType {
    NormalFile = b'0',
    HardLink = b'1',
    SymbolicLink = b'2',
    CharacterSpecial = b'3',
    BlockSpecial = b'4',
    Directory = b'5',
    FIFO = b'6',
    ContiguousFile = b'7',
    GlobalExtendedHeaderWithMetadata = b'g',
    ExtendedHeaderWithMetadataForTheNextFileInTheArchive = b'x',
    VendorSpecificExtention(U8),
}

impl Spec for Header_FileType {
    type Params = ();
    fn read_all<S: ByteSource>(data: &S, _params: Self::Params) -> SResult<(Self, usize)> {
        let mut view = View::from(data); // at 0
        let byte = U8::read_from_view(&mut view, ())?; // at 1
        Ok((match byte.value {
            b'0' | 0 => Header_FileType::NormalFile,
            b'1' => Header_FileType::HardLink,
            b'2' => Header_FileType::SymbolicLink,
            b'3' => Header_FileType::CharacterSpecial,
            b'4' => Header_FileType::BlockSpecial,
            b'5' => Header_FileType::Directory,
            b'6' => Header_FileType::FIFO,
            b'7' => Header_FileType::ContiguousFile,
            b'g' => Header_FileType::GlobalExtendedHeaderWithMetadata,
            b'x' => Header_FileType::ExtendedHeaderWithMetadataForTheNextFileInTheArchive,
            b'A'..=b'Z' => Header_FileType::VendorSpecificExtention(byte),
            rest => return Err(spec_error!(view.cursor.offset; "Reserved filetype used: {rest}")),
        }, view.cursor.offset)) // view.cursor.offset = 1
    }
}

// ----------------------------------------------------------------------
// Ustar extension fields (anonymous spec inside Try)

#[derive(Debug)]
pub struct UStar {
    pub ustar_indicator: Vec<u8>,
    pub ustar_version: Vec<u8>,
    pub owner_user_name: Option<Vec<u8>>,
    pub owner_group_name: Option<Vec<u8>>,
    pub device_major_number: Option<u64>,
    pub device_minor_number: Option<u64>,
    pub filename_prefix: Vec<u8>,
    pub unspecified: Vec<u8>
}

impl Spec for UStar {
    type Params = ();
    fn read_all<S: ByteSource>(data: &S, _params: Self::Params) -> SResult<(Self, usize)> {
        let mut view = View::from(data); // at 0
        let ustar_indicator = TarString::read_from_view(&mut view, 6)?.text.ok_or(spec_error!(view.cursor.offset; "ustar_indicator is required"))?; // at 6 (calculated from TarString)
        assert_spec_eq!(&ustar_indicator, b"ustar");
        let ustar_version = TarString::read_from_view(&mut view, 2)?.text.ok_or(spec_error!(view.cursor.offset; "ustar_version is required"))?; // at 8 (calculated from TarString)
        assert_spec_eq!(&ustar_version, b"00","This file uses a newer version of UStar that is not yet supported");
        let owner_user_name = TarString::read_from_view(&mut view, 32)?.text; // at 40 (calculated from TarString)
        let owner_group_name = TarString::read_from_view(&mut view, 32)?.text; // at 72 (calculated from TarString)
        let device_major_number = Oct::read_from_view(&mut view, None)?.value; // at 80 (calculated from Oct)
        let device_minor_number = Oct::read_from_view(&mut view, None)?.value; // at 88 (calculated from Oct)
        let filename_prefix = TarString::read_from_view(&mut view, 155)?.text.ok_or(spec_error!(view.cursor.offset; "filename_prefix is required"))?; // at 243 (calculated from TarString)
        // unspecified [U8;12] – consume and ignore
        let unspecified = view.consume_n(12)?.into(); // at 255

        Ok((UStar {
            ustar_indicator,
            ustar_version,
            owner_user_name,
            owner_group_name,
            device_major_number,
            device_minor_number,
            filename_prefix,
            unspecified
        }, view.cursor.offset)) // view.cursor.offset = 255
    }
}

#[derive(Debug)]
pub struct Header {
    pub file_mode: Option<u64>,
    pub uid: Option<u64>,
    pub gid: Option<u64>,
    pub file_size: usize,
    pub last_modified: Option<u64>,
    pub check_sum: Option<u64>,
    pub file_type: Header_FileType
,
    pub name_of_linked_file: Option<Vec<u8>>,
    pub filename: Vec<u8>,
}

impl Spec for Header {
    type Params = ();
    fn read_all<S: ByteSource>(data: &S, _params: Self::Params) -> SResult<(Self, usize)> {
        let mut all = View::from(data); // at 0
        let file_path_and_name = TarString::read_from_view(&mut all, 100)?.text.ok_or(spec_error!(all.cursor.offset; "file_path_and_name is required"))?; // at 100 (calculated from TarString)
        let file_mode = Oct::read_from_view(&mut all, None)?.value; // at 108 (calculated from Oct)
        let uid = Oct::read_from_view(&mut all, None)?.value; // at 116 (calculated from Oct)
        let gid = Oct::read_from_view(&mut all, None)?.value; // at 124 (calculated from Oct)
        let file_size = Oct::read_from_view(&mut all, Some(12))?.value.ok_or(spec_error!(all.cursor.offset; "file_size is required"))? as usize; // at 136 (calculated from Oct)
        let last_modified = Oct::read_from_view(&mut all, Some(12))?.value; // at 148 (calculated from Oct)
        let check_sum = Oct::read_from_view(&mut all, None)?.value; // at 156 (calculated from Oct)

        let file_type = Header_FileType::read_from_view(&mut all, ())?; // at 157 (calculated from Header_FileType)

        let name_of_linked_file = TarString::read_from_view(&mut all, 100)?.text; // at 257 (calculated from TarString)

        // unused: Try(UStar, [U8;255])
        // We need to pass params for TryResult<UStar, Vec<u8>>.
        // Fallback reads [U8;255] -> use array! macro.
        let unused = try_spec(
            |view| UStar::read_from_view(view, ()), // at 512 (calculated from UStar)
            |view| array![U8::read_from_view(view, ()); 255], // at 512
            &mut all
        ); // at 512 in both

        // filename calculation
        let filename = match unused {
            either::Either::Left(ustar) => {
                ustar.filename_prefix.iter().chain(&file_path_and_name).cloned().collect()
            },
            either::Either::Right((array, _ustar_error)) => {
                array?;
                file_path_and_name
            }
        };
        if let Some(checksum_value) = check_sum {
            // sum = sum_of_all - sum_of_checksum + 8 * 32
            let actual_sum: u32 = data.peek_n(512)?.iter().map(|&b| b as u32).sum();
            assert_spec_eq!(
                actual_sum as u64,
                checksum_value,
                "Header checksum mismatch"
            );
        }

        Ok((Header {
            file_mode,
            uid,
            gid,
            file_size,
            last_modified,
            check_sum,
            file_type,
            name_of_linked_file,
            filename,
        }, all.cursor.offset)) // all.cursor.offset = 512
    }
}

// ----------------------------------------------------------------------
// TarEntry

#[derive(Debug)]
pub struct TarEntry {
    pub header: Header,
    pub file_data: Vec<u8>,
}

impl Spec for TarEntry {
    type Params = ();
    fn read_all<S: ByteSource>(data: &S, _params: Self::Params) -> SResult<(Self, usize)> {
        let mut view = View::from(data); // at 0
        let header = Header::read_from_view(&mut view, ())?; // at 512 (calculated from Header)
        let file_data = view.consume_n(header.file_size)?.into(); // at 512 + header.file_size
        Ok((TarEntry { header, file_data }, view.cursor.offset)) // view.cursor.offset = 512 + header.file_size
    }
}

// ----------------------------------------------------------------------
// TarFile (the whole archive)

#[derive(Debug)]
pub struct TarFile {
    pub entries: Vec<TarEntry>,
}

impl Spec for TarFile {
    type Params = ();
    fn read_all<S: ByteSource>(data: &S, _params: Self::Params) -> SResult<(Self, usize)> {
        let mut view = View::from(data); // at 0
        let mut entries = Vec::new();
        loop {
            let block = view.peek_n(512)?; // at 0, unchanged
            if block.iter().all(|&b| b == 0) {
                // end‑of‑archive marker (zero block); standard tar has two consecutive zero blocks,
                // we consumed the first one. The second one will be skipped in the next iteration if present.
                break;
            }
            let entry = TarEntry::read_from_view(&mut view, ())?; // at 512 + entry.header.file_size
            // align to next 512‑byte boundary (tar pads file data)
            let modulo = entry.header.file_size % 512;
            view.consume_n((512 - modulo) % 512)?; // at 512 + entry.header.file_size (512 - entry.header.file_size) % 512
            entries.push(entry);
        }
        Ok((TarFile { entries }, view.cursor.offset)) // view.cursor.offset = sum(512 + entry.header.file_size (512 - entry.header.file_size) % 512) for each
    }
}

// ----------------------------------------------------------------------
// entry point

fn main() {
    default_main();
}