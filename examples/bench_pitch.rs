#[inline]
fn denorm_pow64(x: f32) -> f32 {
    let note: f64 = ((x as f64) - 0.5) * 120.0; /* + 69.0 */
    (440.0 * (2.0_f64).powf((note/* - 69.0 */) / 12.0)) as f32
}

#[inline]
fn denorm_pow32(x: f32) -> f32 {
    let note: f32 = ((x as f32) - 0.5) * 120.0; /* + 69.0 */
    440.0 * (2.0_f32).powf((note/* - 69.0 */) / 12.0)
}

fn build_table() -> Vec<f32> {
    let mut v = vec![];
    v.push(denorm_pow32(0.001)); // for the 0
    for x in 1..51 {
        v.push(denorm_pow32((x as f32) / 50.0));
    }
    v
}

#[inline]
fn denorm_interp(tbl: &[f32], x: f32) -> f32 {
    let i = x * 50.0;
    let fract = i.fract();
    let idx = i.floor() as usize;
    //    println!("XX: {} => {} / {}", x, idx, fract);
    let f1 = tbl[idx];
    //    f1
    let f2 = tbl[idx + 1];
    f1 * (1.0 - fract) + f2 * fract
}

// no fract interpolation:
//
// denorm_pow64 Elapsed: 17.83890244s (2027219281437.3845)
// denorm_pow32 Elapsed: 6.688424681s (2027219283489.4597)
// denorm_inter Elapsed: 4.121668955s (1958488898618.3506)

fn main() {
    let ta = std::time::Instant::now();
    let mut res: f64 = 0.0;
    for _i in 0..100000 {
        for x in 1..9999 {
            res += denorm_pow64((x as f32) / 10000.0) as f64;
        }
    }
    println!("denorm_pow64 Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res: f64 = 0.0;
    for _i in 0..100000 {
        for x in 1..9999 {
            res += denorm_pow32((x as f32) / 10000.0) as f64;
        }
    }
    println!("denorm_pow32 Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let itbl = build_table();

    let ta = std::time::Instant::now();
    let mut res: f64 = 0.0;
    for _i in 0..100000 {
        for x in 1..9999 {
            res += denorm_interp(&itbl[..], (x as f32) / 10000.0) as f64;
        }
    }
    println!("denorm_inter Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let mut res1 = 0.0;
    for x in 1..999 {
        res1 += denorm_pow64((x as f32) / 1000.0) as f64;
    }

    let mut res2 = 0.0;
    for x in 1..999 {
        res2 += denorm_interp(&itbl[..], (x as f32) / 1000.0) as f64;
    }

    println!("res1: {}", res1 / 10000.0);
    println!("res2: {}", res2 / 10000.0);
}
