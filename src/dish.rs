//! Plato de Petri: una célula se llena y un sello se tinta si hay prueba.
//! Es la imagen de `evolve` + `prove`, sin escribir archivos.

use crate::{evolve_on, parse_expr, sse, target, would_prove, Champion, BRAIN, XS};
use std::error::Error;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

const WIDTH: usize = 63;
const HEIGHT: usize = 21;
const MORPH: u32 = 8;

pub fn run(steps: u32, lambda: u32, seed: u64, delay_ms: u64) -> Result<(), Box<dyn Error>> {
    let sse0 = sse(&parse_expr(BRAIN)?);
    let _cursor = HideCursor::new();
    let mut prev_h = 0.0_f32;
    let mut history: Vec<(u32, f64)> = Vec::new();
    let mut last: Option<(u32, Champion, f32)> = None;

    evolve_on(steps, lambda, seed, |step, champ, improved| {
        history.push((step, champ.sse));
        let points = sample(champ);
        let h = health(champ.sse, sse0);
        let sealed = would_prove(champ);
        if step == 0 {
            let _ = present(step, champ, sse0, h, "guess", &points, &history, sealed);
            sleep(delay_ms.saturating_mul(4));
            prev_h = h;
            last = Some((step, champ.clone(), h));
            return;
        }
        if !improved {
            return;
        }
        for i in 1..=MORPH {
            let t = i as f32 / MORPH as f32;
            let hi = prev_h + (h - prev_h) * t;
            let st = if champ.sse == 0.0 {
                if sealed {
                    "sealed"
                } else {
                    "fit"
                }
            } else if sealed {
                "proved"
            } else {
                "adapting"
            };
            let _ = present(step, champ, sse0, hi, st, &points, &history, sealed);
            sleep(delay_ms);
        }
        prev_h = h;
        last = Some((step, champ.clone(), h));
        sleep(delay_ms.saturating_mul(2));
    })?;

    if let Some((step, champ, h)) = last {
        let points = sample(&champ);
        let sealed = would_prove(&champ);
        let st = if sealed && champ.sse == 0.0 {
            "sealed"
        } else if sealed {
            "proved"
        } else if champ.sse == 0.0 {
            "fit"
        } else {
            "done"
        };
        let _ = present(step, &champ, sse0, h, st, &points, &history, sealed);
        sleep(delay_ms.saturating_mul(4));
    }

    Ok(())
}

fn sample(champ: &Champion) -> Vec<(i32, f64, f64)> {
    (-XS..=XS)
        .map(|x| {
            let xf = x as f64;
            (x, target(xf), champ.expr.eval(xf))
        })
        .collect()
}

fn health(sse: f64, sse0: f64) -> f32 {
    if !sse.is_finite() {
        return 0.0;
    }
    if sse0 <= 0.0 {
        return 1.0;
    }
    let r = (sse / sse0).clamp(0.0, 1.0);
    1.0 - (r.sqrt() as f32)
}

fn sleep(ms: u64) {
    if ms > 0 {
        thread::sleep(Duration::from_millis(ms));
    }
}

struct HideCursor;

impl HideCursor {
    fn new() -> Self {
        print!("\x1b[?25l\x1b[2J");
        let _ = io::stdout().flush();
        HideCursor
    }
}

impl Drop for HideCursor {
    fn drop(&mut self) {
        print!("\x1b[?25h\x1b[0m\n");
        let _ = io::stdout().flush();
    }
}

struct Canvas {
    w: usize,
    h: usize,
    ch: Vec<char>,
    fg: Vec<u8>,
}

impl Canvas {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            ch: vec![' '; w * h],
            fg: vec![0; w * h],
        }
    }

    fn put(&mut self, x: i32, y: i32, ch: char, color: u8) {
        if x < 0 || y < 0 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.w || y >= self.h {
            return;
        }
        let i = y * self.w + x;
        self.ch[i] = ch;
        self.fg[i] = color;
    }
}

fn present(
    step: u32,
    champ: &Champion,
    sse0: f64,
    h: f32,
    status: &str,
    points: &[(i32, f64, f64)],
    history: &[(u32, f64)],
    sealed: bool,
) -> io::Result<()> {
    let mut c = Canvas::new(WIDTH, HEIGHT);
    draw_dish(&mut c);
    let color = cell_color(h);
    put_cell(&mut c, 16, 10, h, champ.sse, color);
    put_seal(&mut c, 16, 17, sealed, color);
    put_plot(&mut c, 36, 1, 24, 9, points, color);
    put_history(&mut c, 36, 11, 24, 9, history, sse0, color);

    let mut out = String::with_capacity(WIDTH * HEIGHT * 8);
    out.push_str("\x1b[H");
    out.push_str("\x1b[0m  demostrante  ·  petri dish\x1b[K\n");
    for y in 0..c.h {
        let mut last = 255_u8;
        for x in 0..c.w {
            let i = y * c.w + x;
            if c.fg[i] != last {
                if c.fg[i] == 0 {
                    out.push_str("\x1b[0m");
                } else {
                    out.push_str(&format!("\x1b[{}m", c.fg[i]));
                }
                last = c.fg[i];
            }
            out.push(c.ch[i]);
        }
        out.push_str("\x1b[0m\x1b[K\n");
    }
    out.push_str("\x1b[0m");
    let brain = champ.expr.emit();
    let brain = if brain.len() > 42 {
        format!("{}…", &brain[..41])
    } else {
        brain
    };
    out.push_str(&format!(
        "  {status}  ·  step {step}  ·  sse {:>8.1} / {sse0:.0}  ·  {:>3.0}%\x1b[K\n",
        champ.sse,
        h * 100.0
    ));
    out.push_str(&format!("  brain  {brain}\x1b[K\n"));
    out.push_str("  the seal inks only when the proof checks\x1b[K\n");
    let mut stdout = io::stdout();
    stdout.write_all(out.as_bytes())?;
    stdout.flush()
}

fn cell_color(h: f32) -> u8 {
    if h < 0.25 {
        31
    } else if h < 0.55 {
        33
    } else if h < 0.95 {
        32
    } else {
        36
    }
}

fn draw_dish(c: &mut Canvas) {
    let (cx, cy, rx, ry) = (16_i32, 10_i32, 15_i32, 9_i32);
    for y in 0..HEIGHT as i32 {
        for x in 0..34 {
            let dx = (x - cx) as f32 / rx as f32;
            let dy = (y - cy) as f32 / ry as f32;
            let d = dx * dx + dy * dy;
            if (d - 1.0).abs() < 0.08 {
                c.put(x, y, '·', 90);
            }
        }
    }
}

fn put_cell(c: &mut Canvas, cx: i32, cy: i32, h: f32, sse: f64, color: u8) {
    let rx = 4.0 + 5.0 * h;
    let ry = 2.0 + 2.5 * h;
    let fill_r = 0.18 + 0.72 * h;
    for y in (cy - ry.ceil() as i32)..=(cy + ry.ceil() as i32) {
        for x in (cx - rx.ceil() as i32)..=(cx + rx.ceil() as i32) {
            let dx = (x as f32 - cx as f32) / rx.max(0.5);
            let dy = (y as f32 - cy as f32) / ry.max(0.5);
            let d = dx * dx + dy * dy;
            if d > 1.0 {
                continue;
            }
            let membrane = d > 0.68;
            if membrane {
                c.put(x, y, 'o', color);
            } else if d <= fill_r {
                c.put(x, y, ':', color);
            }
        }
    }
    let label = if sse == 0.0 {
        "*".to_string()
    } else {
        format!("{:.0}", (h * 100.0).round())
    };
    let start = cx - (label.len() as i32) / 2;
    for (i, ch) in label.chars().enumerate() {
        c.put(start + i as i32, cy, ch, color);
    }
}

fn put_seal(c: &mut Canvas, cx: i32, cy: i32, sealed: bool, color: u8) {
    let col = if sealed { color } else { 90 };
    let w = 4;
    for x in (cx - w)..=(cx + w) {
        c.put(x, cy - 1, '-', col);
        c.put(x, cy + 1, '-', col);
    }
    c.put(cx - w, cy, '|', col);
    c.put(cx + w, cy, '|', col);
    let label = if sealed { "OK" } else { "??" };
    let start = cx - (label.len() as i32) / 2;
    for (i, ch) in label.chars().enumerate() {
        c.put(start + i as i32, cy, ch, col);
    }
}

fn put_plot(
    c: &mut Canvas,
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    points: &[(i32, f64, f64)],
    color: u8,
) {
    c.put(x0, y0, 't', 90);
    let (mut ymin, mut ymax) = (f64::MAX, f64::MIN);
    for (_, t, p) in points {
        for v in [*t, *p] {
            if v.is_finite() {
                ymin = ymin.min(v);
                ymax = ymax.max(v);
            }
        }
    }
    if !ymin.is_finite() || (ymax - ymin).abs() < 1e-9 {
        ymin = 0.0;
        ymax = 1.0;
    }
    let pad = (ymax - ymin) * 0.08;
    ymin -= pad;
    ymax += pad;
    let xmin = -XS as f64;
    let xmax = XS as f64;

    for &(x, t, p) in points {
        let px = x0 + 2 + (((x as f64 - xmin) / (xmax - xmin)) * (w - 4) as f64).round() as i32;
        let ty = y0 + h
            - 1
            - (((t - ymin) / (ymax - ymin)) * (h - 2) as f64)
                .round()
                .clamp(0.0, (h - 2) as f64) as i32;
        let py = if p.is_finite() {
            y0 + h
                - 1
                - (((p - ymin) / (ymax - ymin)) * (h - 2) as f64)
                    .round()
                    .clamp(0.0, (h - 2) as f64) as i32
        } else {
            y0
        };
        c.put(px, ty, '*', 90);
        if p.is_finite() && py == ty {
            c.put(px, py, '@', color);
        } else if p.is_finite() {
            c.put(px, py, 'o', color);
        }
    }
}

fn put_history(
    c: &mut Canvas,
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    history: &[(u32, f64)],
    sse0: f64,
    color: u8,
) {
    c.put(x0, y0, 'e', 90);
    let inner_w = (w - 3).max(1) as usize;
    let inner_h = (h - 2).max(1);
    for x in 0..inner_w {
        c.put(x0 + 2 + x as i32, y0 + h - 1, '·', 90);
    }
    let cols = crate::bin_history(history, inner_w);
    for (col, v) in cols.iter().enumerate() {
        let Some(sse) = *v else {
            continue;
        };
        let ratio = if !sse.is_finite() {
            1.0
        } else if sse0 <= 0.0 {
            0.0
        } else {
            (sse / sse0).clamp(0.0, 1.0).sqrt()
        };
        let y = y0 + 1 + ((1.0 - ratio) * (inner_h.saturating_sub(1)) as f64).round() as i32;
        let px = x0 + 2 + col as i32;
        let ch = if sse == 0.0 { '@' } else { '*' };
        let fg = if sse == 0.0 { color } else { 90 };
        c.put(px, y, ch, fg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_is_zero_when_sse_is_start() {
        assert!((health(4323.0, 4323.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn health_is_one_when_sse_is_zero() {
        assert!((health(0.0, 4323.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn health_rises_as_sse_drops() {
        let a = health(2000.0, 4323.0);
        let b = health(100.0, 4323.0);
        assert!(b > a);
        assert!(a > 0.0 && b < 1.0);
    }
}
