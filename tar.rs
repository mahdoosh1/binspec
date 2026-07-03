spec TarString(size: Int) {
    // [to_be_called; how_many_times]
    // [type; integer]
    // [|| TypeThatHasParameters(2); 5] is allowed
    // if rust has something that calls a function n times and returns array of return values, that should be used instead of this syntax
    pub let Text = [U8; size-1]; // if rust has string type for ascii, that type should be used instead of arrays
    // call a type means -> read and return value of that type
    let End = U8();
    assert(End == 0 || End == ' ');
}

spec Oct(size = 8) { // implicitly of type Int
    let Digits = TarString(size).Text;
    pub let value = parse_int(Digits, 8);
}

#[align(256)]
spec Header() {
    pub let FilePathAndName = TarString(100).Text;
    pub let FileMode = Oct().value;
    pub let UID = Oct().value;
    pub let GID = Oct().value;
    pub let FileSize = Oct(12).value;
    pub let LastModified = Oct(12).value;
    pub let CheckSum = Oct().value;
    pub let FileType = Enum<U8>(|value| match value { // boilerplate?
        '0' | 0 => NormalFile,
        '1' => HardLink,
        '2' => SymbolicLink,
        '3' => CharacterSpecial,
        '4' => BlockSpecial,
        '5' => Directory,
        '6' => FIFO,
        '7' => ContiguousFile,
        'g' => GlobalExtendedHeaderWithMetadata,
        'x' => ExtendedHeaderWithMetadataForTheNextFileInTheArchive,
        'A'..'Z' => VendorSpecificExtention(value),
        _ => Reserved
    });
    assert(FileType != FileType::Reserved, "Reserved filetype used");
    pub let NameOfLinkedFile = TarString(100).Text;
    pub let Unused = Try( // Try will use the first type, if it fails, it uses the second type
        spec {
            pub let UStarIndicator = TarString(6).Text;
            pub let UStarVersion = TarString(2).Text;
            pub let OwnerUserName = TarString(32).Text;
            pub let OwnerGroupName = TarString(32).Text;
            pub let DeviceMajorNumber = Oct().value;
            pub let DeviceMinorNumber = Oct().value;
            pub let FilenamePrefix = TarString(155).Text;
            pub let Unspecified = [U8; 12];
        },
        [U8; 255]
    ); // Try returns an enum with variants `Ok(type)` or `Fallback(type, error)`.
    // `value as type` -> take `value` as unparsed bytes and parse them as `type`
    assert(
        sum(self as [U8])
        - sum(CheckSum as [U8]) + (CheckSum.size * ' ') // sum of all headers with CheckSum replaced as spaces.
        == CheckSum.value
    );
    // for use outside:
    match Unused {
        Ok(UStar) => 'block: {
            if Ustar.UStarIndicator != b"ustar" {
                pub let filename = FilePathAndName;
                break 'block; // rust quirck, needing a named block
            }
            assert(Ustar.UStarVersion == b"00", "This file uses a newer version of UStar that is not yet supported");
            pub let filename = UStar.FilenamePrefix + FilePathAndName;
        }
        Fallback => {
            pub let filename = FilePathAndName;
        }
    }
}

spec TarEntry() {
    pub let Header = Header();
    pub let FileData = [U8; Header.FileSize];
}

spec File() {
    pub let Entries = [TarEntry];
}