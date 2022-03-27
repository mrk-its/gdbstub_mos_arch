use core::num::NonZeroUsize;

use gdbstub::arch::{Arch, RegId, Registers, SingleStepGdbBehavior};

/// Implements `Arch` for ARMv4T
pub enum MOSArch {}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct MosRegs {
    pub rc: [u8; 32],
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub s: u8,
    pub flags: u8,
}

impl Registers for MosRegs {
    type ProgramCounter = u16;

    fn pc(&self) -> Self::ProgramCounter {
        self.pc
    }

    fn gdb_serialize(&self, mut write_byte: impl FnMut(Option<u8>)) {
        macro_rules! write_bytes {
            ($bytes:expr) => {
                for b in $bytes {
                    write_byte(Some(*b))
                }
            };
        }
        write_bytes!(&self.pc.to_le_bytes());
        write_bytes!(&self.a.to_le_bytes());
        write_bytes!(&self.x.to_le_bytes());
        write_bytes!(&self.y.to_le_bytes());
        write_bytes!(&self.s.to_le_bytes());
        write_bytes!(&(self.flags & 1).to_le_bytes());
        write_bytes!(&((self.flags >> 1) & 1).to_le_bytes());
        write_bytes!(&((self.flags >> 6) & 1).to_le_bytes());
        write_bytes!(&((self.flags >> 7) & 1).to_le_bytes());

        self.rc.iter().for_each(|v| write_byte(Some(*v)));
    }

    fn gdb_deserialize(&mut self, bytes: &[u8]) -> Result<(), ()> {
        self.pc = bytes[0] as u16 + bytes[1] as u16 * 256;
        self.a = bytes[2];
        self.x = bytes[3];
        self.y = bytes[4];
        self.s = bytes[5];

        self.flags &= 0b00111100;
        self.flags |= bytes[6] | bytes[7] * 2 | bytes[8] * 64 + bytes[9] * 128;

        self.rc.iter_mut().enumerate().for_each(|(i, v)| *v = bytes[10 + i]);
        Ok(())
    }
}

#[derive(Debug)]
pub enum MosRegId {
    RC(usize),
    RS(usize),
    PC,
    A,
    X,
    Y,
    S,
    C,
    Z,
    N,
    V,
}

impl RegId for MosRegId {
    fn from_raw_id(id: usize) -> Option<(Self, Option<NonZeroUsize>)> {
        let (reg, size) = match id {
            0 => (MosRegId::PC, 2),
            1 => (MosRegId::A, 1),
            2 => (MosRegId::X, 1),
            3 => (MosRegId::Y, 1),
            4 => (MosRegId::S, 1),
            5 => (MosRegId::C, 1),
            6 => (MosRegId::Z, 1),
            7 => (MosRegId::N, 1),
            8 => (MosRegId::V, 1),
            9..=40 => (MosRegId::RC(id-9), 1),
            41..=56 => (MosRegId::RS(id-41), 2),
            _ => return None,
        };
        return Some((reg, Some(NonZeroUsize::new(size).unwrap())));
    }
}

#[derive(Debug)]
pub enum MosBreakpointKind {
    /// 16-bit Thumb mode breakpoint.
    Regular,
}

impl gdbstub::arch::BreakpointKind for MosBreakpointKind {
    fn from_usize(_kind: usize) -> Option<Self> {
        Some(MosBreakpointKind::Regular)
    }
}

impl Arch for MOSArch {
    type Usize = u16;
    type Registers = MosRegs;
    type RegId = MosRegId;
    type BreakpointKind = MosBreakpointKind;

    fn target_description_xml() -> Option<&'static str> {
        Some(r#"
        <?xml version="1.0"?>
        <!DOCTYPE target SYSTEM "gdb-target.dtd">
        <target version="1.0">
            <architecture>mos</architecture>
            <flags id="flags" size="1">
                <field name="C" start="0" end="0" type="bool" />
                <field name="Z" start="1" end="1" type="bool" />
                <field name="V" start="6" end="6" type="bool" />
                <field name="N" start="7" end="7" type="bool" />
            </flags>
            <groups>
                <group id="1" name="imaginary, 8-bit"></group>
                <group id="2" name="imaginary, 16-bit"></group>
            </groups>
            <feature name="org.gnu.gdb.mos">
                <reg name="PC" bitsize="16" offset="0" regnum="0" generic="pc" />
                <reg name="A" bitsize="8" offset="2" regnum="1" dwarf_regnum="0" />
                <reg name="X" bitsize="8" offset="3" regnum="2" dwarf_regnum="2" />
                <reg name="Y" bitsize="8" offset="4" regnum="3" dwarf_regnum="4" />
                <reg name="S" bitsize="8" offset="5" regnum="4" />
                <reg name="C" bitsize="1" offset="6" regnum="5" />
                <reg name="Z" bitsize="1" offset="7" regnum="6" />
                <reg name="V" bitsize="1" offset="8" regnum="7" />
                <reg name="N" bitsize="1" offset="9" regnum="8" />
                <reg name="RC0" group_id="1" bitsize="8" offset="10" regnum="9" dwarf_regnum="16" />
                <reg name="RC1" group_id="1" bitsize="8" offset="11" regnum="10" dwarf_regnum="18" />
                <reg name="RC2" group_id="1" bitsize="8" offset="12" regnum="11" dwarf_regnum="20" />
                <reg name="RC3" group_id="1" bitsize="8" offset="13" regnum="12" dwarf_regnum="22" />
                <reg name="RC4" group_id="1" bitsize="8" offset="14" regnum="13" dwarf_regnum="24" />
                <reg name="RC5" group_id="1" bitsize="8" offset="15" regnum="14" dwarf_regnum="26" />
                <reg name="RC6" group_id="1" bitsize="8" offset="16" regnum="15" dwarf_regnum="28" />
                <reg name="RC7" group_id="1" bitsize="8" offset="17" regnum="16" dwarf_regnum="30" />
                <reg name="RC8" group_id="1" bitsize="8" offset="18" regnum="17" dwarf_regnum="32" />
                <reg name="RC9" group_id="1" bitsize="8" offset="19" regnum="18" dwarf_regnum="34" />
                <reg name="RC10" group_id="1" bitsize="8" offset="20" regnum="19" dwarf_regnum="36" />
                <reg name="RC11" group_id="1" bitsize="8" offset="21" regnum="20" dwarf_regnum="38" />
                <reg name="RC12" group_id="1" bitsize="8" offset="22" regnum="21" dwarf_regnum="40" />
                <reg name="RC13" group_id="1" bitsize="8" offset="23" regnum="22" dwarf_regnum="42" />
                <reg name="RC14" group_id="1" bitsize="8" offset="24" regnum="23" dwarf_regnum="44" />
                <reg name="RC15" group_id="1" bitsize="8" offset="25" regnum="24" dwarf_regnum="46" />
                <reg name="RC16" group_id="1" bitsize="8" offset="26" regnum="25" dwarf_regnum="48" />
                <reg name="RC17" group_id="1" bitsize="8" offset="27" regnum="26" dwarf_regnum="50" />
                <reg name="RC18" group_id="1" bitsize="8" offset="28" regnum="27" dwarf_regnum="52" />
                <reg name="RC19" group_id="1" bitsize="8" offset="29" regnum="28" dwarf_regnum="54" />
                <reg name="RC20" group_id="1" bitsize="8" offset="30" regnum="29" dwarf_regnum="56" />
                <reg name="RC21" group_id="1" bitsize="8" offset="31" regnum="30" dwarf_regnum="58" />
                <reg name="RC22" group_id="1" bitsize="8" offset="32" regnum="31" dwarf_regnum="60" />
                <reg name="RC23" group_id="1" bitsize="8" offset="33" regnum="32" dwarf_regnum="62" />
                <reg name="RC24" group_id="1" bitsize="8" offset="34" regnum="33" dwarf_regnum="64" />
                <reg name="RC25" group_id="1" bitsize="8" offset="35" regnum="34" dwarf_regnum="66" />
                <reg name="RC26" group_id="1" bitsize="8" offset="36" regnum="35" dwarf_regnum="68" />
                <reg name="RC27" group_id="1" bitsize="8" offset="37" regnum="36" dwarf_regnum="70" />
                <reg name="RC28" group_id="1" bitsize="8" offset="38" regnum="37" dwarf_regnum="72" />
                <reg name="RC29" group_id="1" bitsize="8" offset="39" regnum="38" dwarf_regnum="74" />
                <reg name="RC30" group_id="1" bitsize="8" offset="40" regnum="39" dwarf_regnum="76" />
                <reg name="RC31" group_id="1" bitsize="8" offset="41" regnum="40" dwarf_regnum="78" />
                <reg name="RS0" group_id="2" bitsize="16" offset="10" regnum="41" dwarf_regnum="528" />
                <reg name="RS1" group_id="2" bitsize="16" offset="12" regnum="42" dwarf_regnum="529" />
                <reg name="RS2" group_id="2" bitsize="16" offset="14" regnum="43" dwarf_regnum="530" />
                <reg name="RS3" group_id="2" bitsize="16" offset="16" regnum="44" dwarf_regnum="531" />
                <reg name="RS4" group_id="2" bitsize="16" offset="18" regnum="45" dwarf_regnum="532" />
                <reg name="RS5" group_id="2" bitsize="16" offset="20" regnum="46" dwarf_regnum="533" />
                <reg name="RS6" group_id="2" bitsize="16" offset="22" regnum="47" dwarf_regnum="534" />
                <reg name="RS7" group_id="2" bitsize="16" offset="24" regnum="48" dwarf_regnum="535" />
                <reg name="RS8" group_id="2" bitsize="16" offset="26" regnum="49" dwarf_regnum="536" />
                <reg name="RS9" group_id="2" bitsize="16" offset="28" regnum="50" dwarf_regnum="537" />
                <reg name="RS10" group_id="2" bitsize="16" offset="30" regnum="51" dwarf_regnum="538" />
                <reg name="RS11" group_id="2" bitsize="16" offset="32" regnum="52" dwarf_regnum="539" />
                <reg name="RS12" group_id="2" bitsize="16" offset="34" regnum="53" dwarf_regnum="540" />
                <reg name="RS13" group_id="2" bitsize="16" offset="36" regnum="54" dwarf_regnum="541" />
                <reg name="RS14" group_id="2" bitsize="16" offset="38" regnum="55" dwarf_regnum="542" />
                <reg name="RS15" group_id="2" bitsize="16" offset="40" regnum="56" dwarf_regnum="543" />
            </feature>
        </target>
        "#)
    }

    #[inline(always)]
    fn single_step_gdb_behavior() -> SingleStepGdbBehavior {
        SingleStepGdbBehavior::Optional
    }
}
