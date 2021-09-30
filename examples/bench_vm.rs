// Benchmark Program:
//
// i = input
// output = if i > 0.5 { sin((i - 0.5) * TAU) } else { (i + 0.3) * 1.1 * i }

#[inline(never)]
fn variant_native_dsp(i: f32) -> f32 {
    if i > 0.5 {
        ((i - 0.5) * std::f32::consts::TAU).sin()
    } else {
        (i + 0.3) * 1.1 * i
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Op {
    BrIfGt(f32, u8),
    BrIfLe(f32, u8),
    Br(u8),
    Sin,
    Add(f32),
    Sub(f32),
    Mul(f32),
    Mul2,
    Div(f32),
    Dup,
    Push(f32),
}

pub struct StackVM {
    stack:  [f32; 100],
    sp:     usize,
}

impl StackVM {
    pub fn new() -> Self {
        Self {
            stack: [0.0; 100],
            sp: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, i: f32) {
        self.stack[self.sp] = i;
        self.sp += 1;
    }

    #[inline]
    pub fn pop(&mut self) -> f32 {
        self.sp -= 1;
        self.stack[self.sp]
    }

    #[inline]
    pub fn exec(&mut self, prog: &[Op], i: f32) -> f32 {
        self.push(i);

        let mut ip = 0;
        while ip < prog.len() {
            let op = prog[ip];
            //d// println!("OP={:?} SP={}", op, self.sp);
            //d// for s in 0..self.sp {
            //d//     println!(" S[{}]={}", s, self.stack[s]);
            //d// }
            match op {
                Op::BrIfLe(arg, offs) => {
                    let x = self.pop();
                    if x <= arg {
                        ip += offs as usize;
                    }
                },
                Op::BrIfGt(arg, offs) => {
                    let x = self.pop();
                    if x > arg {
                        ip += offs as usize;
                    }
                },
                Op::Br(offs) => { ip += offs as usize; },
                Op::Sin      =>  { let a = self.pop(); self.push(a.sin()); },
                Op::Add(arg) =>  { let a = self.pop(); self.push(a + arg); },
                Op::Sub(arg) =>  { let a = self.pop(); self.push(a - arg); },
                Op::Mul(arg) =>  { let a = self.pop(); self.push(a * arg); },
                Op::Div(arg) =>  { let a = self.pop(); self.push(a / arg); },
                Op::Mul2      => { let (a, b) = (self.pop(), self.pop()); self.push(a * b); },
                Op::Push(arg) =>  { self.push(arg); },
                Op::Dup       =>  { let x = self.pop(); self.push(x); self.push(x); },
            }

            ip += 1;
        }

        self.pop()
    }
}

// output = if i > 0.5 { sin((i - 0.5) * TAU) } else { (i + 0.3) * 1.1 * i }
const PROG : [Op; 10] = [
    Op::Dup, // x = i
    Op::BrIfLe(0.5, 4),       // if x > 0.5
    Op::Sub(0.5),
    Op::Mul(std::f32::consts::TAU),
    Op::Sin,
    Op::Br(4),   // skip else branch

    Op::Dup,     // copy i
    Op::Add(0.3),     // i1 + 0.3
    Op::Mul(1.1),     // i1 * 1.1
    Op::Mul2,    // i1 * i
];

pub fn variant_stack_vm(vm: &mut StackVM, i: f32) -> f32 {
    vm.exec(&PROG[..], i)
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum RegOp {
    BrIfGt(f32, u8, u8),
    BrIfLe(f32, u8, u8),
    Br(u8),
    Sin(u8, u8),
    Add(f32, u8, u8),
    Sub(f32, u8, u8),
    Mul(f32, u8, u8),
    Mul2(u8, u8, u8),
    Div(f32, u8, u8),
    Set(f32, u8),
    Fo1,
    Fo2,
    Fo3,
    Fo4,
    Fo5,
    Fo6,
    Fo7,
    Fo8,
    Fo9,
    Fo11,
    Fo12,
    Fo13,
    Fo14,
    Fo15,
    Fo16,
    Fo17,
    Fo18,
    Fo19,
    Fo21,
    Fo22,
    Fo23,
    Fo24,
    Fo25,
    Fo26,
    Fo27,
    Fo28,
    Fo29,
    Fo31,
    Fo32,
    Fo33,
    Fo34,
    Fo35,
    Fo36,
    Fo37,
    Fo38,
    Fo39,
}

pub struct RegVM {
    regs: [f32; 100],
}

impl RegVM {
    pub fn new() -> Self {
        Self {
            regs: [0.0; 100],
        }
    }

    #[inline(never)]
    pub fn add(&mut self, x: f32, a: usize, r: usize) {
        self.regs[r] = self.regs[a] + x;
    }

    #[inline(never)]
    pub fn mul(&mut self, x: f32, a: usize, r: usize) {
        self.regs[r] = self.regs[a] * x;
    }

    #[inline(never)]
    pub fn mul2(&mut self, b: usize, a: usize, r: usize) {
        self.regs[r] = self.regs[a] * self.regs[b];
    }

    #[inline(never)]
    pub fn sin(&mut self, a: usize, r: usize) {
        self.regs[r] = self.regs[a].sin();
    }

    #[inline(never)]
    pub fn exec_dir(&mut self, prog: &[RegOp], i: f32) -> f32 {
        self.regs[0] = i;

        if self.regs[0] > 0.5 {
            self.add(-0.5, 0, 0);
            self.mul(std::f32::consts::TAU, 0, 0);
            self.sin(0, 0);
        } else {
            self.add(0.3, 0, 1);
            self.mul(1.1, 1, 1);
            self.mul2(0, 1, 0);
        }

        self.regs[0]
    }

    #[inline]
    pub fn exec(&mut self, prog: &[RegOp], i: f32) -> f32 {
        self.regs[0] = i;

        let mut ip = 0;
        while ip < prog.len() {
            let op = prog[ip];
            //d// println!("OP={:?} SP={}", op, self.sp);
            //d// for s in 0..self.sp {
            //d//     println!(" S[{}]={}", s, self.stack[s]);
            //d// }
            match op {
                RegOp::BrIfLe(arg, offs, r_a) => {
                    if self.regs[r_a as usize] <= arg {
                        ip += offs as usize;
                    }
                },
                RegOp::BrIfGt(arg, offs, r_a) => {
                    if self.regs[r_a as usize] > arg {
                        ip += offs as usize;
                    }
                },
                RegOp::Br(offs) => { ip += offs as usize; },
                RegOp::Sin(r_a, r_r) => {
                    self.regs[r_r as usize] = self.regs[r_a as usize].sin();
                },
                RegOp::Add(arg, r_a, r_r) => {
                    self.regs[r_r as usize] = self.regs[r_a as usize] + arg;
                },
                RegOp::Sub(arg, r_a, r_r) => {
                    self.regs[r_r as usize] = self.regs[r_a as usize] - arg;
                },
                RegOp::Mul(arg, r_a, r_r) => {
                    self.regs[r_r as usize] = self.regs[r_a as usize] * arg;
                },
                RegOp::Div(arg, r_a, r_r) => {
                    self.regs[r_r as usize] = self.regs[r_a as usize] / arg;
                },
                RegOp::Mul2(r_a, r_b, r_r)=> {
                    let (a, b) = (self.regs[r_a as usize], self.regs[r_b as usize]);
                    self.regs[r_r as usize]= a * b;
                },
                RegOp::Set(arg, r_r) => {
                    self.regs[r_r as usize] = arg;
                },
                RegOp::Fo1 => { self.regs[0] += 0.1; },
                RegOp::Fo2 => { self.regs[0] += 0.2; },
                RegOp::Fo3 => { self.regs[0] += 0.3; },
                RegOp::Fo4 => { self.regs[0] += 0.4; },
                RegOp::Fo5 => { self.regs[0] += 0.5; },
                RegOp::Fo6 => { self.regs[0] += 0.6; },
                RegOp::Fo7 => { self.regs[0] += 0.7; },
                RegOp::Fo8 => { self.regs[0] += 0.8; },
                RegOp::Fo9 => { self.regs[0] += 0.9; },
                RegOp::Fo11 => { self.regs[1] += 0.1; },
                RegOp::Fo12 => { self.regs[1] += 0.2; },
                RegOp::Fo13 => { self.regs[1] += 0.3; },
                RegOp::Fo14 => { self.regs[1] += 0.4; },
                RegOp::Fo15 => { self.regs[1] += 0.5; },
                RegOp::Fo16 => { self.regs[1] += 0.6; },
                RegOp::Fo17 => { self.regs[1] += 0.7; },
                RegOp::Fo18 => { self.regs[1] += 0.8; },
                RegOp::Fo19 => { self.regs[1] += 0.9; },
                RegOp::Fo21 => { self.regs[2] += 0.1; },
                RegOp::Fo22 => { self.regs[2] += 0.2; },
                RegOp::Fo23 => { self.regs[2] += 0.3; },
                RegOp::Fo24 => { self.regs[2] += 0.4; },
                RegOp::Fo25 => { self.regs[2] += 0.5; },
                RegOp::Fo26 => { self.regs[2] += 0.6; },
                RegOp::Fo27 => { self.regs[2] += 0.7; },
                RegOp::Fo28 => { self.regs[2] += 0.8; },
                RegOp::Fo29 => { self.regs[2] += 0.9; },
                RegOp::Fo31 => { self.regs[3] += 0.1; },
                RegOp::Fo32 => { self.regs[3] += 0.2; },
                RegOp::Fo33 => { self.regs[3] += 0.3; },
                RegOp::Fo34 => { self.regs[3] += 0.4; },
                RegOp::Fo35 => { self.regs[3] += 0.5; },
                RegOp::Fo36 => { self.regs[3] += 0.6; },
                RegOp::Fo37 => { self.regs[3] += 0.7; },
                RegOp::Fo38 => { self.regs[3] += 0.8; },
                RegOp::Fo39 => { self.regs[3] += 0.9; },
            }

            ip += 1;
        }

        self.regs[0]
    }
}

// output =
//     if i > 0.5 { sin((i - 0.5) * TAU) }
//     else       { (i + 0.3) * 1.1 * i }
const REG_PROG : [RegOp; 8] = [
    RegOp::BrIfLe(0.5, 4, 0), // if i > 0.5
    RegOp::Sub(0.5, 0, 0),
    RegOp::Mul(std::f32::consts::TAU, 0, 0),
    RegOp::Sin(0, 0),
    RegOp::Br(5),   // skip else branch

    RegOp::Add(0.3, 0, 1), // i1 + 0.3
    RegOp::Mul(1.1, 1, 1), // i1 * 1.1
    RegOp::Mul2(0, 1, 0),  // i1 * i
];

pub fn variant_reg_vm(vm: &mut RegVM, i: f32) -> f32 {
    vm.exec(&REG_PROG[..], i)
}

pub fn variant_reg_vm_dir(vm: &mut RegVM, i: f32) -> f32 {
    vm.exec_dir(&REG_PROG[..], i)
}

pub fn run() {
    let max = 100000000;

    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for i in 0..max {
        let x = i as f32 / (max as f32);
        res += variant_native_dsp(x);
    }

    println!("native Elapsed: {:?} ({})",
             std::time::Instant::now().duration_since(ta), res);

    let mut vm = StackVM::new();

    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for i in 0..max {
        let x = i as f32 / (max as f32);
        res += variant_stack_vm(&mut vm, x);
    }

    println!("stk Elapsed: {:?} ({})",
             std::time::Instant::now().duration_since(ta), res);

    let mut rvm = RegVM::new();

    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for i in 0..max {
        let x = i as f32 / (max as f32);
        res += variant_reg_vm(&mut rvm, x);
    }

    println!("reg Elapsed: {:?} ({})",
             std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for i in 0..max {
        let x = i as f32 / (max as f32);
        res += variant_reg_vm_dir(&mut rvm, x);
    }

    println!("rdi Elapsed: {:?} ({})",
             std::time::Instant::now().duration_since(ta), res);
}

pub fn main() {
    let mut vm  = StackVM::new();
    let mut rvm = RegVM::new();
    println!("NATIVE(0.2) => {}", variant_native_dsp(0.2));
    println!("NATIVE(0.8) => {}", variant_native_dsp(0.8));
    println!("VM    (0.2) => {}", variant_stack_vm(&mut vm, 0.2));
    println!("VM    (0.8) => {}", variant_stack_vm(&mut vm, 0.8));
    println!("REG VM(0.2) => {}", variant_reg_vm(&mut rvm, 0.2));
    println!("REG VM(0.8) => {}", variant_reg_vm(&mut rvm, 0.8));
    println!("RDI VM(0.2) => {}", variant_reg_vm_dir(&mut rvm, 0.2));
    println!("RDI VM(0.8) => {}", variant_reg_vm_dir(&mut rvm, 0.8));
    run();
    run();
    run();
}
