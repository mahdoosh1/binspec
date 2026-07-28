spec TarString(size: USIZE) {
    // [to_be_called; how_many_times]
    // [type; integer]
    // [|| TypeThatHasParameters(2); 5] is allowed
    // if rust has something that calls a function n times and returns array of return values, that should be used instead of this syntax
    hide let string = [U8; size-1]; // if rust has string type for ascii, that type should be used instead of arrays
    // call a type means -> read and return value of that type
    hide let end = U8();
    assert!(end == 0 || end == b' '); // not rust's assert
    hide let end = string.find(0).or(string.find(b' '));
    let text = end.and_then(|index| string[..index]);
}

spec Oct(size: USIZE = 8) {
    hide let digits = TarString(size).text;
    let value: Option<U64> = digits.and_then(|digits| parse(digits, 8));
}

spec UStar {
    let ustar_indicator = TarString(6).text;
    assert(ustar_indicator == b"ustar");
    let ustar_version = TarString(2).text;
    assert(ustar.ustar_version == b"00", "This file uses a newer version of UStar that is not yet supported");
    let owner_user_name = TarString(32).text;
    let owner_group_name = TarString(32).text;
    let device_major_number = Oct().value;
    let device_minor_number = Oct().value;
    let filename_prefix = TarString(155).text;
    let unspecified = [U8; 12];
}

spec Header() {
    hide let file_path_and_name = TarString(100).text;
    let file_mode = Oct().value;
    let UID = Oct().value;
    let GID = Oct().value;
    let file_size = Oct(12).value;
    let last_modified = Oct(12).value;
    let check_sum = Oct();
    enum FileType: U8 {
        b'0' => NormalFile,
        b'1' => HardLink,
        b'2' => SymbolicLink,
        b'3' => CharacterSpecial,
        b'4' => BlockSpecial,
        b'5' => Directory,
        b'6' => FIFO,
        b'7' => ContiguousFile,
        b'g' => GlobalExtendedHeaderWithMetadata,
        b'x' => ExtendedHeaderWithMetadataForTheNextFileInTheArchive,
        b'A'..=b'Z' => VendorSpecificExtention,
        rest => error("Reserved filetype used: {rest}")
    }
    let file_type = FileType();
    let name_of_linked_file = TarString(100).text;
    let unused = Try( // Try will use the first type, if it fails, it uses the second type
        Ustar,
        [U8; 255]
    ); // Try returns an enum with variants `Ok(type)` or `Fallback(type, error)` or it raises an error if fallback type didn't work.
    // `value as type` -> take `value` as unparsed bytes and parse them as `type`
    assert(
        sum(self as [U8])
        - sum(check_sum as [U8]) + (check_sum.size * b' ') // sum of all headers with CheckSum replaced as spaces.
        == check_sum.value
    );
    // for use outside:
    let filename = match unused {
        Ok(ustar) => {
            if ustar.ustar_indicator != b"ustar" {
                break file_path_and_name
            }
            
            uStar.filename_prefix + file_path_and_name
        }
        Fallback => {
            file_path_and_name
        }
    }
}

spec TarEntry() {
    let header = Header();
    let file_data = [U8; Header.FileSize];
}
spec File() {
    let entries = [[U8 = 0; Header.size]? TarEntry];
}