use crate::cpu::cpu_core::CpuMode;
use crate::cpu::memory::memory_core::Memory;
use crate::types::{decode_program_line, ProgramLine, Word};
use anyhow::Result;
use bitflags::bitflags;
use std::cmp::max;
use std::fmt::{Display, Formatter};
use std::{fmt, fs};

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum WordSize {
    W32,
    W64,
}

#[derive(Debug)]
pub enum Endian {
    Little,
    Big,
}

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum ABI {
    SystemV,
    Other,
}

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum ELFType {
    Relocatable,
    Executable,
    Shared,
    Core,
    Unknown,
}

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum ISA {
    RISCV,
    X86,
    ARM,
    MIPS,
    PPC,
    SPARC,
    OTHER,
}

#[derive(Debug)]
pub struct ELFHeader {
    pub word_size: WordSize,
    pub endian: Endian,
    pub version: u8,
    pub abi: ABI,
    pub elf_type: ELFType,
    pub isa: ISA,
    pub entry_point: u64,
    pub program_header_table_offset: u64,
    pub section_header_table_offset: u64,
    pub header_size: u16,
    pub program_header_size: u16,
    pub program_header_count: u16,
    pub section_header_size: u16,
    pub section_header_count: u16,
    pub section_header_string_table_index: u16,
}

impl Display for ELFHeader {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "ELF Header")?;
        writeln!(f, "  Word Size: {:?}", self.word_size)?;
        writeln!(f, "  Endian: {:?}", self.endian)?;
        writeln!(f, "  Version: {}", self.version)?;
        writeln!(f, "  ABI: {:?}", self.abi)?;
        writeln!(f, "  ELF Type: {:?}", self.elf_type)?;
        writeln!(f, "  ISA: {:?}", self.isa)?;
        writeln!(f, "  Entry Point: {:#x}", self.entry_point)?;
        writeln!(
            f,
            "  Program Header Table Offset: {:#x}",
            self.program_header_table_offset
        )?;
        writeln!(
            f,
            "  Section Header Table Offset: {:#x}",
            self.section_header_table_offset
        )?;
        writeln!(f, "  Header Size: {}", self.header_size)?;
        writeln!(f, "  Program Header Size: {}", self.program_header_size)?;
        writeln!(f, "  Program Header Count: {}", self.program_header_count)?;
        writeln!(f, "  Section Header Size: {}", self.section_header_size)?;
        writeln!(f, "  Section Header Count: {}", self.section_header_count)?;
        writeln!(
            f,
            "  Section Header String Table Index: {}",
            self.section_header_string_table_index
        )
    }
}

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
enum SectionType {
    SHT_NULL,
    SHT_PROGBITS,
    SHT_SYMTAB,
    SHT_STRTAB,
    SHT_RELA,
    SHT_HASH,
    SHT_DYNAMIC,
    SHT_NOTE,
    SHT_NOBITS,
    SHT_REL,
    SHT_SHLIB,
    SHT_DYNSYM,
    SHT_INIT_ARRAY,
    SHT_FINI_ARRAY,
    SHT_PREINIT_ARRAY,
    SHT_GROUP,
    SHT_SYMTAB_SHNDX,
    SHT_NUM,
    SHT_LOOS,
}

bitflags! {
    struct SectionFlags: u64 {
        const SHF_WRITE            = 0x1;
        const SHF_ALLOC            = 0x2;
        const SHF_EXECINSTR        = 0x4;
        const SHF_MERGE            = 0x10;
        const SHF_STRINGS          = 0x20;
        const SHF_INFO_LINK        = 0x40;
        const SHF_LINK_ORDER       = 0x80;
        const SHF_OS_NONCONFORMING = 0x100;
        const SHF_GROUP            = 0x200;
        const SHF_TLS              = 0x400;
        const SHF_MASKOS           = 0x0ff00000;
        const SHF_MASKPROC         = 0xf0000000;
        const SHF_ORDERED          = 0x40000000;
        const SHF_EXCLUDE          = 0x80000000;
    }
}

impl fmt::Display for SectionFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();

        if self.contains(SectionFlags::SHF_WRITE) {
            flags.push("WRITE");
        }
        if self.contains(SectionFlags::SHF_ALLOC) {
            flags.push("ALLOC");
        }
        if self.contains(SectionFlags::SHF_EXECINSTR) {
            flags.push("EXECINSTR");
        }
        if self.contains(SectionFlags::SHF_MERGE) {
            flags.push("MERGE");
        }
        if self.contains(SectionFlags::SHF_STRINGS) {
            flags.push("STRINGS");
        }
        if self.contains(SectionFlags::SHF_INFO_LINK) {
            flags.push("INFO_LINK");
        }
        if self.contains(SectionFlags::SHF_LINK_ORDER) {
            flags.push("LINK_ORDER");
        }
        if self.contains(SectionFlags::SHF_OS_NONCONFORMING) {
            flags.push("OS_NONCONFORMING");
        }
        if self.contains(SectionFlags::SHF_GROUP) {
            flags.push("GROUP");
        }
        if self.contains(SectionFlags::SHF_TLS) {
            flags.push("TLS");
        }
        if self.contains(SectionFlags::SHF_MASKOS) {
            flags.push("MASKOS");
        }
        if self.contains(SectionFlags::SHF_MASKPROC) {
            flags.push("MASKPROC");
        }
        if self.contains(SectionFlags::SHF_ORDERED) {
            flags.push("ORDERED");
        }
        if self.contains(SectionFlags::SHF_EXCLUDE) {
            flags.push("EXCLUDE");
        }

        if flags.is_empty() {
            write!(f, "NONE")
        } else {
            write!(f, "{}", flags.join(" | "))
        }
    }
}

#[allow(unused)]
pub struct Section {
    name: String,
    section_type: SectionType,
    flags: SectionFlags,
    addr: usize,
    offset: usize,
    size: usize,
    data: Vec<u8>,
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Section:")?;
        writeln!(f, "  Name: {}", self.name)?;
        writeln!(f, "  Type: {:?}", self.section_type)?;
        writeln!(f, "  Flags: {}", self.flags)?;
        writeln!(f, "  Address: 0x{:x}", self.addr)?;
        writeln!(f, "  Offset: 0x{:x}", self.offset)?;
        writeln!(f, "  Size: 0x{:x}", self.size)
    }
}

#[derive(Debug, PartialEq)]
pub enum ProgramHeaderType {
    Load,
    Dynamic,
    Interp,
    Note,
    Shlib,
    Phdr,
    Tls,
    Loos,
    Hios,
    Loproc,
    Hiproc,
    Unknown(u32),
}

#[derive(Debug)]
pub struct ProgramHeader {
    header_type: ProgramHeaderType,
    flags: u64,
    segment_offset: u64,
    virtual_address: u64,
    physical_address: u64,
    file_size: u64,
    memory_size: u64,
    alignment: u64,
    data: Vec<u8>,
}

pub struct ElfFile {
    pub header: ELFHeader,
    pub program_headers: Vec<ProgramHeader>,
    pub section_headers: Vec<Section>,
}

impl Display for ProgramHeader {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "Program Header")?;
        writeln!(f, "  Type: {:?}", self.header_type)?;
        writeln!(f, "  Flags: {:#x}", self.flags)?;
        writeln!(f, "  Segment Offset: {:#x}", self.segment_offset)?;
        writeln!(f, "  Virtual Address: {:#x}", self.virtual_address)?;
        writeln!(f, "  Physical Address: {:#x}", self.physical_address)?;
        writeln!(f, "  File Size: {:#x}", self.file_size)?;
        writeln!(f, "  Memory Size: {:#x}", self.memory_size)?;
        writeln!(f, "  Alignment: {:#x}", self.alignment)
    }
}

pub struct ProgramFile {
    pub entry_point: u64,
    pub program_memory_offset: u64,
    pub lines: Vec<ProgramLine>,
    pub program_size: u64,
    pub end_of_data_addr: u64,
}

pub fn decode_file(path: &str) -> ElfFile {
    let file = fs::read(path).unwrap();

    let magic_value = u32::from_be_bytes(
        file.iter()
            .take(4)
            .cloned()
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap(),
    );

    let expected_magic_value = u32::from_be_bytes([0x7F, b'E', b'L', b'F']);

    assert_eq!(magic_value, expected_magic_value);

    let word_size = match file[0x4] {
        1 => WordSize::W32,
        2 => WordSize::W64,
        _ => panic!("Invalid word size"),
    };

    let endian = match file[0x5] {
        1 => Endian::Little,
        2 => Endian::Big,
        _ => panic!("Invalid endian"),
    };

    let version = file[0x6];

    let abi = match file[0x7] {
        0 => ABI::SystemV,
        _ => ABI::Other,
    };

    // Ignore ABI Version

    // Ignore padding

    let elf_type = match u16::from_le_bytes(file[0x10..0x12].try_into().unwrap()) {
        0 => ELFType::Unknown,
        1 => ELFType::Relocatable,
        2 => ELFType::Executable,
        3 => ELFType::Shared,
        4 => ELFType::Core,
        _ => panic!(
            "{}",
            format!(
                "Invalid ELF type: {:#x}",
                u16::from_le_bytes(file[0x10..0x12].try_into().unwrap())
            )
        ),
    };

    let isa = match u16::from_le_bytes(file[0x12..0x14].try_into().unwrap()) {
        0xF3 => ISA::RISCV,
        0x3E => ISA::X86,
        0xB7 => ISA::ARM,
        0x8C => ISA::MIPS,
        0x14 => ISA::PPC,
        0x2B => ISA::SPARC,
        _ => ISA::OTHER,
    };

    // Ignore ABI Version

    let entry_point = read_file_word_size(&file, 0, word_size, 0x18, 0x18);

    let program_header_table_offset = read_file_word_size(&file, 0, word_size, 0x1C, 0x20);

    let section_header_table_offset = read_file_word_size(&file, 0, word_size, 0x20, 0x28);

    // Ignore flags

    let header_size = read_file_u16(&file, word_size, 0x28, 0x34);

    let program_header_size = read_file_u16(&file, word_size, 0x2A, 0x36);

    let program_header_count = read_file_u16(&file, word_size, 0x2C, 0x38);

    let section_header_size = read_file_u16(&file, word_size, 0x2E, 0x3A);

    let section_header_count = read_file_u16(&file, word_size, 0x30, 0x3C);

    let section_header_string_table_index = read_file_u16(&file, word_size, 0x32, 0x3E);

    let elf_header = ELFHeader {
        word_size,
        endian,
        version,
        abi,
        elf_type,
        isa,
        entry_point,
        program_header_table_offset,
        section_header_table_offset,
        header_size,
        program_header_size,
        program_header_count,
        section_header_size,
        section_header_count,
        section_header_string_table_index,
    };

    let mut program_headers = vec![];
    let mut section_headers = vec![];

    for i in 0..elf_header.program_header_count {
        let offset = (elf_header.program_header_table_offset
            + i as u64 * elf_header.program_header_size as u64) as usize;

        let program_header_type =
            match u32::from_le_bytes(file[offset..(offset + 4)].try_into().unwrap()) {
                1 => ProgramHeaderType::Load,
                2 => ProgramHeaderType::Dynamic,
                3 => ProgramHeaderType::Interp,
                4 => ProgramHeaderType::Note,
                5 => ProgramHeaderType::Shlib,
                6 => ProgramHeaderType::Phdr,
                7 => ProgramHeaderType::Tls,
                0x60000000 => ProgramHeaderType::Loos,
                0x6FFFFFFF => ProgramHeaderType::Hios,
                0x70000000 => ProgramHeaderType::Loproc,
                0x7FFFFFFF => ProgramHeaderType::Hiproc,
                other => ProgramHeaderType::Unknown(other),
            };

        let flags = read_file_u32(&file, word_size, offset + 0x18, offset + 0x4) as u64;

        let segment_offset = read_file_word_size(&file, offset, word_size, 0x4, 0x8);

        let virtual_address = read_file_word_size(&file, offset, word_size, 0x8, 0x10);

        let physical_address = read_file_word_size(&file, offset, word_size, 0xC, 0x18);

        let file_size = read_file_word_size(&file, offset, word_size, 0x10, 0x20);

        let memory_size = read_file_word_size(&file, offset, word_size, 0x14, 0x28);

        let alignment = read_file_word_size(&file, offset, word_size, 0x1C, 0x30);

        let segment_data = if program_header_type == ProgramHeaderType::Load {
            &file[segment_offset as usize..(segment_offset + file_size) as usize]
        } else {
            &[]
        };

        let program_header = ProgramHeader {
            header_type: program_header_type,
            flags,
            segment_offset,
            virtual_address,
            physical_address,
            file_size,
            memory_size,
            alignment,
            data: segment_data.to_vec(),
        };

        program_headers.push(program_header);
    }

    // Name string table

    let offset = (elf_header.section_header_table_offset
        + elf_header.section_header_string_table_index as u64
            * elf_header.section_header_size as u64) as usize;

    let shstrtab_offset = read_file_word_size(&file, offset, word_size, 0x10, 0x18) as usize;

    for i in 0..elf_header.section_header_count {
        let offset = (elf_header.section_header_table_offset
            + i as u64 * elf_header.section_header_size as u64) as usize;

        let section_header_name_offset =
            u32::from_le_bytes(file[(offset)..(offset + 0x4)].try_into().unwrap()) as usize;

        let string_start = shstrtab_offset + section_header_name_offset;

        // Find the end of the string (null terminator)
        let string_end = file[string_start..]
            .iter()
            .position(|&x| x == 0)
            .map(|pos| string_start + pos)
            .unwrap_or(file.len());

        // Extract the section header name
        let section_header_name = std::str::from_utf8(&file[string_start..string_end]).unwrap();

        let section_type_raw = u32::from_le_bytes(
            file[(offset + 0x4)..(offset + 0x4 + 0x4)]
                .try_into()
                .unwrap(),
        );

        let section_type = match section_type_raw {
            0 => SectionType::SHT_NULL,
            1 => SectionType::SHT_PROGBITS,
            2 => SectionType::SHT_SYMTAB,
            3 => SectionType::SHT_STRTAB,
            4 => SectionType::SHT_RELA,
            5 => SectionType::SHT_HASH,
            6 => SectionType::SHT_DYNAMIC,
            7 => SectionType::SHT_NOTE,
            8 => SectionType::SHT_NOBITS,
            9 => SectionType::SHT_REL,
            10 => SectionType::SHT_SHLIB,
            11 => SectionType::SHT_DYNSYM,
            14 => SectionType::SHT_INIT_ARRAY,
            15 => SectionType::SHT_FINI_ARRAY,
            16 => SectionType::SHT_PREINIT_ARRAY,
            17 => SectionType::SHT_GROUP,
            18 => SectionType::SHT_SYMTAB_SHNDX,
            0x6ffffffa => SectionType::SHT_LOOS,
            _ => SectionType::SHT_NUM,
        };

        let section_flags_raw = read_file_word_size(&file, offset, word_size, 0x08, 0x08);

        let section_flags = SectionFlags::from_bits_truncate(section_flags_raw);

        let section_addr = read_file_word_size(&file, offset, word_size, 0x0C, 0x10) as usize;

        let section_offset = read_file_word_size(&file, offset, word_size, 0x10, 0x18) as usize;

        let section_size = read_file_word_size(&file, offset, word_size, 0x14, 0x20) as usize;

        let section: &[u8] = if section_flags.contains(SectionFlags::SHF_ALLOC)
            && section_type != SectionType::SHT_NOBITS
        {
            &file[section_offset..(section_offset + section_size)]
        } else {
            &[]
        };

        let section_header = Section {
            name: section_header_name.to_owned(),
            section_type,
            flags: section_flags,
            addr: section_addr,
            offset: section_offset,
            size: section_size,
            data: section.to_vec(),
        };

        section_headers.push(section_header);
    }
    ElfFile {
        header: elf_header,
        program_headers,
        section_headers,
    }
}

fn read_file_u16(file: &[u8], word_size: WordSize, position_32: usize, position_64: usize) -> u16 {
    let position = match word_size {
        WordSize::W32 => position_32,
        WordSize::W64 => position_64,
    };

    u16::from_le_bytes(file[(position)..(position + 0x2)].try_into().unwrap())
}

fn read_file_u32(file: &[u8], word_size: WordSize, position_32: usize, position_64: usize) -> u32 {
    let position = match word_size {
        WordSize::W32 => position_32,
        WordSize::W64 => position_64,
    };

    u32::from_le_bytes(file[(position)..(position + 0x4)].try_into().unwrap())
}

fn read_file_word_size(
    file: &[u8],
    offset: usize,
    word_size: WordSize,
    position_32: usize,
    position_64: usize,
) -> u64 {
    match word_size {
        WordSize::W32 => u32::from_le_bytes(
            file[(offset + position_32)..(offset + position_32 + 0x4)]
                .try_into()
                .unwrap(),
        ) as u64,
        WordSize::W64 => u64::from_le_bytes(
            file[(offset + position_64)..(offset + position_64 + 0x8)]
                .try_into()
                .unwrap(),
        ),
    }
}

pub fn load_program_to_memory(
    elf: ElfFile,
    memory: &mut dyn Memory,
    mode: CpuMode,
) -> Result<ProgramFile> {
    let mut program: Vec<ProgramLine> = vec![];
    let mut text_section_addr = 0;
    let mut text_section_size = 0;
    let mut end_of_data_addr = 0;

    for program in elf.program_headers {
        if program.header_type == ProgramHeaderType::Load {
            // Load the segment into memory
            let segment_address = program.virtual_address as usize;

            for (i, byte) in program.data.iter().enumerate() {
                memory
                    .write_mem_u8((segment_address + i) as u64, *byte)
                    .unwrap();
            }
        }
    }

    for section in elf.section_headers {
        if section.flags.contains(SectionFlags::SHF_ALLOC)
            && section.section_type != SectionType::SHT_NOBITS
        {
            for (i, byte) in section.data.iter().take(section.size).enumerate() {
                memory
                    .write_mem_u8((section.addr + i) as u64, *byte)
                    .unwrap();
            }
        }

        if section.flags.contains(SectionFlags::SHF_ALLOC)
            || section.flags.contains(SectionFlags::SHF_EXECINSTR)
            || section.flags.contains(SectionFlags::SHF_WRITE)
        {
            end_of_data_addr = max(end_of_data_addr, section.addr + section.size);
        }

        if section.name == ".text" {
            text_section_addr = section.addr;
            text_section_size = section.size;

            let mut pc = 0;
            while pc < section.size {
                let instruction =
                    u32::from_le_bytes(section.data[pc..(pc + 4)].try_into().unwrap());
                pc += 4;

                let decoded_instruction = decode_program_line(Word(instruction), mode);
                match decoded_instruction {
                    Ok(decoded_instruction) => {
                        program.push(decoded_instruction);
                    }
                    Err(e) => {
                        println!("Error decoding instruction: {}", e);
                    }
                }
            }
        }
    }

    Ok(ProgramFile {
        entry_point: elf.header.entry_point,
        program_memory_offset: text_section_addr as u64,
        program_size: text_section_size as u64,
        lines: program,
        end_of_data_addr: end_of_data_addr as u64,
    })
}

pub fn decode_program_from_binary(binary: &[u32], mode: CpuMode) -> Result<Vec<ProgramLine>> {
    Ok(binary
        .iter()
        .map(|word| decode_program_line(Word(*word), mode).unwrap())
        .collect())
}
