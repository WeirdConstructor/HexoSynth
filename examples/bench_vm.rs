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
            }

            ip += 1;
        }

        self.regs[0]
    }
}

// output = if i > 0.5 { sin((i - 0.5) * TAU) } else { (i + 0.3) * 1.1 * i }
const REG_PROG : [RegOp; 8] = [
    RegOp::BrIfLe(0.5, 4, 0), // if i > 0.5
    RegOp::Sub(0.5, 0, 0),
    RegOp::Mul(std::f32::consts::TAU, 0, 0),
    RegOp::Sin(0, 0),
    RegOp::Br(5),   // skip else branch

//    RegOp::Mul2(0, 1, 4),  // i1 * i
//    RegOp::Mul2(0, 1, 4),  // i1 * i
    RegOp::Add(0.3, 0, 1), // i1 + 0.3
    RegOp::Mul(1.1, 1, 1), // i1 * 1.1
    RegOp::Mul2(0, 1, 0),  // i1 * i
];

pub fn variant_reg_vm(vm: &mut RegVM, i: f32) -> f32 {
    vm.exec(&REG_PROG[..], i)
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
    run();
    run();
    run();
}
