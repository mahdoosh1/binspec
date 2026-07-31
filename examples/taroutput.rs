use binspec::byte_source::ByteSource;
use binspec::errors::SResult;
use binspec::specs::{U8, try_spec};
use binspec::trys::{Left, Right, TryString, get_string};
use binspec::view::View;
use binspec::{array, assert_spec_eq, create_spec, default_main, spec_error};

fn parse_int(bytes: &[u8], radix: u32) -> Option<u64> {
    let s = std::str::from_utf8(bytes).ok()?;
    u64::from_str_radix(s.trim_end_matches(&[' ', '\0'][..]), radix).ok()
}

// ----------------------------------------------------------------------
// TarString

#[derive(Debug)]
pub struct TarString {
    pub text: Option<TryString>,
}

create_spec!(TarString(data, size: usize) {
    let mut view = View::from(data);
    let field_bytes = view.consume_n(size)?;
    let end = field_bytes.iter().position(|&b| b == 0 || b == b' ').unwrap_or(size);
    let trimmed = &field_bytes[..end];
    let text = (!trimmed.is_empty())
        .then(|| get_string(trimmed));
    Ok((
        TarString { text },
        view.offset()
    ))
});

// ----------------------------------------------------------------------
// Oct – octal numeric field

#[derive(Debug)]
pub struct Oct {
    pub value: Option<u64>,
}

create_spec!(Oct(data, size: Option<usize>) {
    let mut view = View::from(data);
    let size = size.unwrap_or(8);

    let digits = TarString::read_from_view(&mut view, size)?.text;
    let value = digits
        .and_then(|a| match a {
            Left(text) => parse_int(text.as_bytes(), 8),
            Right((vec, _)) => parse_int(&vec, 8),
        }
    );

    Ok((
        Oct { value },
        view.offset()
    ))
});

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

create_spec!(Header_FileType(data) {
    let mut view = View::from(data); // at 0
    let byte = U8::read_from_view(&mut view, ())?; // at 1
    let value = match byte.value {
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
        rest => return Err(spec_error!(view.offset(); "Reserved filetype used: {rest}")),
    };
    Ok((value, view.offset())) // view.offset() = 1
});

// ----------------------------------------------------------------------
// Ustar extension fields (anonymous spec inside Try)

#[derive(Debug)]
pub struct UStar {
    pub ustar_indicator: String,
    pub ustar_version: String,
    pub owner_user_name: Option<TryString>,
    pub owner_group_name: Option<TryString>,
    pub device_major_number: Option<u64>,
    pub device_minor_number: Option<u64>,
    pub filename_prefix: String,
    pub unspecified: Vec<u8>,
}

create_spec!(UStar(data) {
    let mut view = View::from(data); // at 0
    let ustar_indicator = TarString::read_from_view(&mut view, 6)?
        .text
        .ok_or(spec_error!(view.offset(); "ustar_indicator is required"))?
        .left()
        .ok_or(spec_error!(view.offset(); "ustar_indicator must be a valid string"))?; // at 6 (calculated from TarString)
    assert_spec_eq!(ustar_indicator.as_bytes(), b"ustar");
    let ustar_version = TarString::read_from_view(&mut view, 2)?
        .text
        .ok_or(spec_error!(view.offset(); "ustar_version is required"))?
        .left()
        .ok_or(spec_error!(view.offset(); "ustar_version must be a valid string"))?; // at 8 (calculated from TarString)
    assert_spec_eq!(ustar_version.as_bytes(), b"00","This file uses a newer version of UStar that is not yet supported");
    let owner_user_name = TarString::read_from_view(&mut view, 32)?.text; // at 40 (calculated from TarString)
    let owner_group_name = TarString::read_from_view(&mut view, 32)?.text; // at 72 (calculated from TarString)
    let device_major_number = Oct::read_from_view(&mut view, None)?.value; // at 80 (calculated from Oct)
    let device_minor_number = Oct::read_from_view(&mut view, None)?.value; // at 88 (calculated from Oct)
    let filename_prefix = TarString::read_from_view(&mut view, 155)?
        .text
        .ok_or(spec_error!(view.offset(); "filename_prefix is required"))?
        .left()
        .ok_or(spec_error!(view.offset(); "filename_prefix must be a valid string"))?; // at 243 (calculated from TarString)
    // unspecified [U8;12] – consume and ignore
    let unspecified = Vec::from(view.consume_n(12)?); // at 255

    Ok((
        UStar {
            ustar_indicator,
            ustar_version,
            owner_user_name,
            owner_group_name,
            device_major_number,
            device_minor_number,
            filename_prefix,
            unspecified
        },
        view.offset()
    )) // view.offset() = 255
});

#[derive(Debug)]
pub struct Header {
    pub file_mode: Option<u64>,
    pub uid: Option<u64>,
    pub gid: Option<u64>,
    pub file_size: usize,
    pub last_modified: Option<u64>,
    pub check_sum: Option<u64>,
    pub file_type: Header_FileType,
    pub name_of_linked_file: Option<TryString>,
    pub filename: TryString,
}

create_spec!(Header(data) {
    let mut all = View::from(data); // at 0
    let file_path_and_name = TarString::read_from_view(&mut all, 100)?
        .text
        .ok_or(spec_error!(all.offset(); "file_path_and_name is required"))?; // at 100 (calculated from TarString)
    let file_mode = Oct::read_from_view(&mut all, None)?.value; // at 108 (calculated from Oct)
    let uid = Oct::read_from_view(&mut all, None)?.value; // at 116 (calculated from Oct)
    let gid = Oct::read_from_view(&mut all, None)?.value; // at 124 (calculated from Oct)
    let file_size = Oct::read_from_view(&mut all, Some(12))?
        .value
        .ok_or(spec_error!(all.offset(); "file_size is required"))? as usize; // at 136 (calculated from Oct)
    let last_modified = Oct::read_from_view(&mut all, Some(12))?.value; // at 148 (calculated from Oct)
    let check_sum = Oct::read_from_view(&mut all, None)?.value; // at 156 (calculated from Oct)

    let file_type = Header_FileType::read_from_view(&mut all, ())?; // at 157 (calculated from Header_FileType)

    let name_of_linked_file = TarString::read_from_view(&mut all, 100)?.text; // at 257 (calculated from TarString)

    // unused: Try(UStar, [U8;255])
    // We need to pass params for TryResult<UStar, Vec<u8>>.
    // Fallback reads [U8;255] -> use array! macro.
    let unused = try_spec(
        |view| UStar::read_from_view(view, ()), // at 512 (calculated from UStar)
        |view| array![U8 = view; 255], // at 512
        &mut all
    ); // at 512 in both

    // filename calculation
    let filename = match unused {
        Left(ustar) => {
            file_path_and_name.map_left(|suffix_str| format!("{}{}", ustar.filename_prefix, suffix_str))
        },
        Right((result, _ustar_error)) => {
            result?;
            file_path_and_name
        }
    };
    if let Some(checksum_value) = check_sum {
        let header_bytes = data.peek_n(512)?; // &[u8]
        let total_sum: u32 = header_bytes.iter().map(|&b| b as u32).sum();

        // Sum the actual checksum field bytes (positions 148..156)
        let checksum_bytes_sum: u32 = header_bytes[148..156].iter().map(|&b| b as u32).sum();

        // Replace the checksum field bytes with spaces (8 * 32)
        let actual_sum = total_sum - checksum_bytes_sum + 8 * 32;

        assert_spec_eq!(
            actual_sum as u64,
            checksum_value,
            "Header checksum mismatch"
        );
    }

    Ok((
        Header {
            file_mode,
            uid,
            gid,
            file_size,
            last_modified,
            check_sum,
            file_type,
            name_of_linked_file,
            filename,
        },
        all.offset()
    )) // all.offset() = 512
});

// ----------------------------------------------------------------------
// TarEntry

#[derive(Debug)]
pub struct TarEntry {
    pub header: Header,
    pub file_data: Vec<u8>,
}

create_spec!(TarEntry(data) {
    let mut view = View::from(data);
    let header = Header::read_from_view(&mut view, ())?; // size = 512 (calculated from Header)
    let file_data = view.consume_n(header.file_size)?.into(); // at 512 + header.file_size
    Ok((
        TarEntry { header, file_data },
        view.offset()
    )) // view.offset() = 512 + header.file_size
});

// ----------------------------------------------------------------------
// TarFile (the whole archive)

#[derive(Debug)]
pub struct File {
    pub entries: Vec<TarEntry>,
}

create_spec!(File(data) {
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
    Ok((
        File { entries },
        view.offset()
    )) // view.offset() = sum(512 + entry.header.file_size (512 - entry.header.file_size) % 512) for each
});

// ----------------------------------------------------------------------
// entry point

fn main() {
    default_main::<File>();
}
