use core::arch::x86_64::{_MM_FLUSH_ZERO_ON, _MM_SET_FLUSH_ZERO_MODE};

#[inline]
pub fn to_bits(x: f32) -> u32 {
    unsafe { ::std::mem::transmute::<f32, u32>(x) }
}

#[inline]
pub fn from_bits(x: u32) -> f32 {
    unsafe { ::std::mem::transmute::<u32, f32>(x) }
}

/// Base 2 logarithm.
#[inline]
pub fn log2(x: f32) -> f32 {
    let mut y = to_bits(x) as f32;
    y *= 1.1920928955078125e-7_f32;
    y - 126.94269504_f32
}

/// Raises 2 to a floating point power.
#[inline]
pub fn pow2(p: f32) -> f32 {
    let clipp = if p < -126.0 { -126.0_f32 } else { p };
    let v = ((1 << 23) as f32 * (clipp + 126.94269504_f32)) as u32;
    from_bits(v)
}

/// Raises a number to a floating point power.
#[inline]
pub fn pow(x: f32, p: f32) -> f32 {
    pow2(p * log2(x))
}

#[inline]
pub fn myfun1(x: f32, v: f32) -> f32 {
    if v > 0.75 {
        let xsq1 = x.sqrt();
        let xsq = xsq1.sqrt();
        let v = (v - 0.75) * 4.0;
        xsq1 * (1.0 - v) + xsq * v
    } else if v > 0.5 {
        let xsq = x.sqrt();
        let v = (v - 0.5) * 4.0;
        x * (1.0 - v) + xsq * v
    } else if v > 0.25 {
        let xx = x * x;
        let v = (v - 0.25) * 4.0;
        x * v + xx * (1.0 - v)
    } else {
        let xx = x * x;
        let xxxx = xx * xx;
        let v = v * 4.0;
        xx * v + xxxx * (1.0 - v)
    }
}

#[inline]
pub fn myfun2(x: f32, v: f32) -> f32 {
    (x).powf(0.25 * v + (1.0 - v) * 4.0)
}

fn main() {
    unsafe {
        _MM_SET_FLUSH_ZERO_MODE(_MM_FLUSH_ZERO_ON);
    }

    let ta = std::time::Instant::now();
    let mut res: f32 = 0.0;
    for i in 0..100000 {
        let v = i as f32 / 100000.0;
        for i in 0..1000 {
            let y = i as f32 / 1000.0;
            res += myfun1(y, v);
        }
    }

    println!("t1 Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res: f32 = 0.0;
    for i in 0..100000 {
        let v = i as f32 / 100000.0;
        for i in 0..1000 {
            let y = i as f32 / 1000.0;
            res += myfun2(y, v);
        }
    }

    println!("t1_b Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res: f32 = 0.0;
    for _i in 0..1000000 {
        for i in 0..1000 {
            //            let x = (i as f32 / 1000.0);
            //            let xx = x * x;
            //            let xxx = xx * xx;
            //            let xsq = x.sqrt().sqrt();
            //            res += 0.3 * xxx + 0.9 * xsq;
            let y = i as f32 / 1000.0;
            res += pow(y, 0.1);
        }
    }

    println!("t2 Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res: f32 = 0.0;
    for _i in 0..1000000 {
        for i in 0..1000 {
            let y = i as f32 / 1000.0;
            res += y.powf(0.1);
        }
    }

    println!("t3 Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res: f32 = 0.0;
    for _i in 0..1000000 {
        for i in 0..1000 {
            let y = i as f32 / 1000.0;
            let y = y.sqrt().sqrt();
            res += y;
        }
    }

    println!("t4 Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);
}
