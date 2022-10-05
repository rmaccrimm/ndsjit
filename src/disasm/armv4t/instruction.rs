#[derive(Copy, Clone, Debug)]
/// Registers available for instruction operands. Not all will necessarily be used. Names roughly
/// follow the convention used in the ARM architecture manuals (Rd, Rn, Rm, etc.)
/// TODO - maybe these should just be integers?
pub struct Registers {
    dest: Option<Register>,
    rn: Option<Register>,
    rm: Option<Register>,
    shift: Option<Register>,
    lo: Option<Register>,
    hi: Option<Register>,
}

#[derive(Copy, Clone, Debug)]
pub struct ImmValue(i32);

#[derive(Copy, Clone, Debug)]
pub enum Cond {
    EQ,
    NE,
    CS,
    CC,
    MI,
    PL,
    VS,
    VC,
    HI,
    LS,
    GE,
    LT,
    GT,
    LE,
    None,
}

#[derive(Copy, Clone, Debug)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    SP,
    LR,
    PC,
    FLAGS,
}

#[derive(Copy, Clone, Debug)]
pub enum DataProcOp {
    ADC,
    ADD,
    ADR,
    AND,
    BIC,
    CMN,
    EOR,
    MOV,
    MVN,
    ORN,
    ORR,
    RSB,
    RSC,
    SBC,
    SUB,
    TEQ,
    TST,
}

#[derive(Copy, Clone, Debug)]
pub enum ShiftOp {
    Logical,
    Arithmetic,
}

#[derive(Copy, Clone, Debug)]
pub enum ShiftDirection {
    Left,
    Right,
}

#[derive(Copy, Clone, Debug)]
pub struct DataProc {
    op: DataProcOp,
    regs: Registers,
    imm: Option<ImmValue>,
}

#[derive(Copy, Clone, Debug)]
pub struct Shift {
    op: ShiftOp,
    dir: ShiftDirection,
    regs: Registers,
    imm: Option<ImmValue>,
}

#[derive(Copy, Clone, Debug)]
pub enum LoadStoreOp {
    Load,
    Store,
}

#[derive(Copy, Clone, Debug)]
pub struct LoadStore {
    op: LoadStoreOp,
    regs: Registers,
}

#[derive(Copy, Clone, Debug)]
pub struct LoadStoreMultiple {}

#[derive(Copy, Clone, Debug)]
pub enum StatusRegOp {
    MSR,
    MRS,
}

#[derive(Copy, Clone, Debug)]
pub struct StatusRegAccess {
    op: StatusRegOp,
    regs: Registers,
    imm: ImmValue,
}

#[derive(Copy, Clone, Debug)]
pub struct Branch {
    link: bool,
    exchange: bool,
    regs: Registers,
}

#[derive(Copy, Clone, Debug)]
pub struct Multiply {
    regs: Registers,
}
