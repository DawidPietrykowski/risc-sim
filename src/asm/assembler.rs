use crate::isa::types::{decode_program_line, Word};
use std::fmt::{Display, Formatter};
use std::{fmt, fs};

struct ELF {}

#[derive(Debug, Clone)]
enum WordSize {
    W32,
    W64,
}

#[derive(Debug)]
enum Endian {
    Little,
    Big,
}

#[derive(Debug)]
enum ABI {
    SystemV,
    Other,
}

#[derive(Debug)]
enum ELFType {
    Relocatable,
    Executable,
    Shared,
    Core,
    Unknown,
}

#[derive(Debug)]
enum ISA {
    RISCV,
    X86,
    ARM,
    MIPS,
    PPC,
    SPARC,
    OTHER,
}

#[derive(Debug)]
struct ELFHeader {
    word_size: WordSize,
    endian: Endian,
    version: u8,
    abi: ABI,
    elf_type: ELFType,
    isa: ISA,
    entry_point: u64,
    program_header_table_offset: u64,
    section_header_table_offset: u64,
    header_size: u16,
    program_header_size: u16,
    program_header_count: u16,
    section_header_size: u16,
    section_header_count: u16,
    section_header_string_table_index: u16,
}

impl Display for ELFHeader {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "ELF Header\n")?;
        write!(f, "  Word Size: {:?}\n", self.word_size)?;
        write!(f, "  Endian: {:?}\n", self.endian)?;
        write!(f, "  Version: {}\n", self.version)?;
        write!(f, "  ABI: {:?}\n", self.abi)?;
        write!(f, "  ELF Type: {:?}\n", self.elf_type)?;
        write!(f, "  ISA: {:?}\n", self.isa)?;
        write!(f, "  Entry Point: {:#x}\n", self.entry_point)?;
        write!(
            f,
            "  Program Header Table Offset: {:#x}\n",
            self.program_header_table_offset
        )?;
        write!(
            f,
            "  Section Header Table Offset: {:#x}\n",
            self.section_header_table_offset
        )?;
        write!(f, "  Header Size: {}\n", self.header_size)?;
        write!(f, "  Program Header Size: {}\n", self.program_header_size)?;
        write!(f, "  Program Header Count: {}\n", self.program_header_count)?;
        write!(f, "  Section Header Size: {}\n", self.section_header_size)?;
        write!(f, "  Section Header Count: {}\n", self.section_header_count)?;
        write!(
            f,
            "  Section Header String Table Index: {}\n",
            self.section_header_string_table_index
        )
    }
}

#[derive(Debug)]
enum ProgramHeaderType {
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
    Unknown,
}

#[derive(Debug)]
struct ProgramHeader {
    header_type: ProgramHeaderType,
    flags: u32,
    segment_offset: u64,
    virtual_address: u64,
    physical_address: u64,
    file_size: u64,
    memory_size: u64,
    alignment: u64,
}

impl ProgramHeader {
    fn new(
        header_type: ProgramHeaderType,
        flags: u32,
        segment_offset: u64,
        virtual_address: u64,
        physical_address: u64,
        file_size: u64,
        memory_size: u64,
        alignment: u64,
    ) -> Self {
        ProgramHeader {
            header_type,
            flags,
            segment_offset,
            virtual_address,
            physical_address,
            file_size,
            memory_size,
            alignment,
        }
    }
}

impl Display for ProgramHeader {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Program Header\n")?;
        write!(f, "  Type: {:?}\n", self.header_type)?;
        write!(f, "  Flags: {:#x}\n", self.flags)?;
        write!(f, "  Segment Offset: {:#x}\n", self.segment_offset)?;
        write!(f, "  Virtual Address: {:#x}\n", self.virtual_address)?;
        write!(f, "  Physical Address: {:#x}\n", self.physical_address)?;
        write!(f, "  File Size: {:#x}\n", self.file_size)?;
        write!(f, "  Memory Size: {:#x}\n", self.memory_size)?;
        write!(f, "  Alignment: {:#x}\n", self.alignment)
    }
}

pub fn decode_file() {
    let file = fs::read("notes/test").unwrap();
    let start = 0;

    let magic_value = u32::from_be_bytes(
        file.iter()
            .take(4)
            .cloned()
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap(),
    );

    let expected_magic_value = u32::from_be_bytes([0x7F, 'E' as u8, 'L' as u8, 'F' as u8]);

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

    let entry_point: u64 = match word_size {
        WordSize::W32 => u32::from_le_bytes(file[0x18..0x1C].try_into().unwrap()) as u64,
        WordSize::W64 => u64::from_le_bytes(file[0x18..0x20].try_into().unwrap()),
    };

    let program_header_table_offset: u64 = match word_size {
        WordSize::W32 => u32::from_le_bytes(file[0x1C..0x20].try_into().unwrap()) as u64,
        WordSize::W64 => u64::from_le_bytes(file[0x20..0x28].try_into().unwrap()),
    };

    let section_header_table_offset: u64 = match word_size {
        WordSize::W32 => u32::from_le_bytes(file[0x20..0x24].try_into().unwrap()) as u64,
        WordSize::W64 => u64::from_le_bytes(file[0x28..0x30].try_into().unwrap()),
    };

    // Ignore flags

    let header_size: u16 = match word_size {
        WordSize::W32 => u16::from_le_bytes(file[0x28..0x2A].try_into().unwrap()),
        WordSize::W64 => u16::from_le_bytes(file[0x34..0x36].try_into().unwrap()),
    };

    let program_header_size: u16 = match word_size {
        WordSize::W32 => u16::from_le_bytes(file[0x2A..0x2C].try_into().unwrap()),
        WordSize::W64 => u16::from_le_bytes(file[0x36..0x38].try_into().unwrap()),
    };

    let program_header_count: u16 = match word_size {
        WordSize::W32 => u16::from_le_bytes(file[0x2C..0x2E].try_into().unwrap()),
        WordSize::W64 => u16::from_le_bytes(file[0x38..0x3A].try_into().unwrap()),
    };

    let section_header_size: u16 = match word_size {
        WordSize::W32 => u16::from_le_bytes(file[0x2E..0x30].try_into().unwrap()),
        WordSize::W64 => u16::from_le_bytes(file[0x3A..0x3C].try_into().unwrap()),
    };

    let section_header_count: u16 = match word_size {
        WordSize::W32 => u16::from_le_bytes(file[0x30..0x32].try_into().unwrap()),
        WordSize::W64 => u16::from_le_bytes(file[0x3C..0x3E].try_into().unwrap()),
    };

    let section_header_string_table_index: u16 = match word_size {
        WordSize::W32 => u16::from_le_bytes(file[0x32..0x34].try_into().unwrap()),
        WordSize::W64 => u16::from_le_bytes(file[0x3E..0x40].try_into().unwrap()),
    };

    let elf_header = ELFHeader {
        word_size: word_size.clone(),
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

    println!("{}", elf_header);

    for i in 0..elf_header.program_header_count {
        let offset = (elf_header.program_header_table_offset
            + i as u64 * elf_header.program_header_size as u64) as usize;

        let program_header_type =
            match u32::from_le_bytes(file[offset..(offset + 4)].try_into().unwrap()) {
                0 => ProgramHeaderType::Load,
                1 => ProgramHeaderType::Dynamic,
                2 => ProgramHeaderType::Interp,
                3 => ProgramHeaderType::Note,
                4 => ProgramHeaderType::Shlib,
                5 => ProgramHeaderType::Phdr,
                6 => ProgramHeaderType::Tls,
                0x60000000 => ProgramHeaderType::Loos,
                0x6FFFFFFF => ProgramHeaderType::Hios,
                0x70000000 => ProgramHeaderType::Loproc,
                0x7FFFFFFF => ProgramHeaderType::Hiproc,
                0x70000003 => ProgramHeaderType::Unknown,
                _ => panic!(
                    "Invalid program header type {:#x}",
                    u32::from_le_bytes(file[offset..(offset + 4)].try_into().unwrap())
                ),
            };

        let flags = match word_size {
            WordSize::W32 => u32::from_le_bytes(
                file[(offset + 0x18)..(offset + 0x18 + 0x4)]
                    .try_into()
                    .unwrap(),
            ),
            WordSize::W64 => u32::from_le_bytes(
                file[(offset + 0x4)..(offset + 0x4 + 0x8)]
                    .try_into()
                    .unwrap(),
            ),
        };

        let segment_offset = match word_size {
            WordSize::W32 => u32::from_le_bytes(
                file[(offset + 0x4)..(offset + 0x4 + 0x4)]
                    .try_into()
                    .unwrap(),
            ) as u64,
            WordSize::W64 => u64::from_le_bytes(
                file[(offset + 0x8)..(offset + 0x8 + 0x8)]
                    .try_into()
                    .unwrap(),
            ),
        };

        let virtual_address = match word_size {
            WordSize::W32 => u32::from_le_bytes(
                file[(offset + 0x8)..(offset + 0x8 + 0x4)]
                    .try_into()
                    .unwrap(),
            ) as u64,
            WordSize::W64 => u64::from_le_bytes(
                file[(offset + 0x10)..(offset + 0x10 + 0x8)]
                    .try_into()
                    .unwrap(),
            ),
        };

        let physical_address = match word_size {
            WordSize::W32 => u32::from_le_bytes(
                file[(offset + 0xC)..(offset + 0xC + 0x4)]
                    .try_into()
                    .unwrap(),
            ) as u64,
            WordSize::W64 => u64::from_le_bytes(
                file[(offset + 0x18)..(offset + 0x18 + 0x8)]
                    .try_into()
                    .unwrap(),
            ),
        };

        let file_size = match word_size {
            WordSize::W32 => u32::from_le_bytes(
                file[(offset + 0x10)..(offset + 0x10 + 0x4)]
                    .try_into()
                    .unwrap(),
            ) as u64,
            WordSize::W64 => u64::from_le_bytes(
                file[(offset + 0x20)..(offset + 0x20 + 0x8)]
                    .try_into()
                    .unwrap(),
            ),
        };

        let memory_size = match word_size {
            WordSize::W32 => u32::from_le_bytes(
                file[(offset + 0x14)..(offset + 0x14 + 0x4)]
                    .try_into()
                    .unwrap(),
            ) as u64,
            WordSize::W64 => u64::from_le_bytes(
                file[(offset + 0x28)..(offset + 0x28 + 0x8)]
                    .try_into()
                    .unwrap(),
            ),
        };

        let alignment = match word_size {
            WordSize::W32 => u32::from_le_bytes(
                file[(offset + 0x1C)..(offset + 0x1C + 0x4)]
                    .try_into()
                    .unwrap(),
            ) as u64,
            WordSize::W64 => u64::from_le_bytes(
                file[(offset + 0x30)..(offset + 0x30 + 0x8)]
                    .try_into()
                    .unwrap(),
            ),
        };

        let program_header = ProgramHeader::new(
            program_header_type,
            flags,
            segment_offset,
            virtual_address,
            physical_address,
            file_size,
            memory_size,
            alignment,
        );

        println!("{}", program_header);
    }

    // Name string table

    let offset = (elf_header.section_header_table_offset
        + elf_header.section_header_string_table_index as u64
            * elf_header.section_header_size as u64) as usize;

    let shstrtab_offset = match word_size {
        WordSize::W32 => u32::from_le_bytes(
            file[(offset + 0x10)..(offset + 0x10 + 0x4)]
                .try_into()
                .unwrap(),
        ) as u64,
        WordSize::W64 => u64::from_le_bytes(
            file[(offset + 0x18)..(offset + 0x18 + 0x8)]
                .try_into()
                .unwrap(),
        ),
    } as usize;

    println!("Section Header String Table Offset: {:#x}", shstrtab_offset);

    for i in 0..elf_header.section_header_count {
        let offset = (elf_header.section_header_table_offset
            + i as u64 * elf_header.section_header_size as u64) as usize;

        let section_header_name_offset =
            u32::from_le_bytes(file[(offset)..(offset + 0x4)].try_into().unwrap()) as usize;

        let string_start = shstrtab_offset + section_header_name_offset;

        println!("Section Header Name Offset: {:#x}", string_start);

        // Find the end of the string (null terminator)
        let string_end = file[string_start..]
            .iter()
            .position(|&x| x == 0)
            .map(|pos| string_start + pos)
            .unwrap_or(file.len());

        // Extract the section header name
        let section_header_name = std::str::from_utf8(&file[string_start..string_end]).unwrap();

        println!("Section Header Name: {}", section_header_name);

        if section_header_name == ".text" {
            let section_offset = match word_size {
                WordSize::W32 => u32::from_le_bytes(
                    file[(offset + 0x10)..(offset + 0x10 + 0x4)]
                        .try_into()
                        .unwrap(),
                ) as u64,
                WordSize::W64 => u64::from_le_bytes(
                    file[(offset + 0x18)..(offset + 0x18 + 0x8)]
                        .try_into()
                        .unwrap(),
                ),
            } as usize;

            let section_size = match word_size {
                WordSize::W32 => u32::from_le_bytes(
                    file[(offset + 0x14)..(offset + 0x14 + 0x4)]
                        .try_into()
                        .unwrap(),
                ) as u64,
                WordSize::W64 => u64::from_le_bytes(
                    file[(offset + 0x20)..(offset + 0x20 + 0x8)]
                        .try_into()
                        .unwrap(),
                ),
            } as usize;

            let section = &file[section_offset..(section_offset + section_size)];

            println!("Section Size: {:#x}", section_size);

            let mut pc = 0;
            while pc < section_size {
                let instruction = u32::from_le_bytes(section[pc..(pc + 4)].try_into().unwrap());
                println!("{:#010x}: {:#034b}", pc, instruction);
                pc += 4;

                let decoded_instruction = decode_program_line(Word(instruction)).unwrap();
                println!("{:?}", decoded_instruction);
            }
        }
    }
}
