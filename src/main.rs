//! Constructor que se reescribe sólo si hay prueba de que mejora.
//!
//! Darwin busca (`evolve`). Gödel sella: `--spawn` / `--write` de un cerebro
//! nuevo exigen un certificado verificable en los 11 puntos del evaluador
//! congelado. `spawn` sin búsqueda copia, y no afirma mejora.

mod dish;

use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::{SystemTime, UNIX_EPOCH};

const GENERATION: u32 = 0;
const LINEAGE: &str = "0";
pub(crate) const BRAIN: &str = "0";
const PARENT_BRAIN: &str = "";
const PROOF: &str = "";

const GENOME: &[(&str, &str)] = &[
    ("Cargo.toml", include_str!("../Cargo.toml")),
    ("src/main.rs", include_str!("main.rs")),
    ("src/dish.rs", include_str!("dish.rs")),
    ("cell.svg", include_str!("../cell.svg")),
    ("README.md", include_str!("../README.md")),
    (".gitignore", include_str!("../.gitignore")),
    ("LICENSE", include_str!("../LICENSE")),
];

const MAX_DEPTH: usize = 7;
const DEFAULT_STEPS: u32 = 120;
const DEFAULT_LAMBDA: u32 = 30;
const DEFAULT_DISH_SEED: u64 = 7;
pub(crate) const XS: i32 = 5;

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        None | Some("help") | Some("-h") | Some("--help") => help(),
        Some("identity") | Some("id") => {
            if let Err(e) = identity() {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Some("eval") => {
            let x = args.next();
            if let Err(e) = eval_cmd(x.as_deref()) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Some("prove") | Some("prueba") => {
            let cand = args.next();
            if let Err(e) = prove_cmd(cand.as_deref()) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Some("genome") => print_genome(),
        Some("evolve") => {
            if let Err(e) = evolve_cmd(args.collect()) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Some("spawn") => {
            let dest = args.next().unwrap_or_else(|| {
                eprintln!("uso: demostrante spawn <directorio> [--build] [--force]");
                process::exit(2);
            });
            let mut force = false;
            let mut build = false;
            for flag in args {
                match flag.as_str() {
                    "--force" => force = true,
                    "--build" => build = true,
                    other => {
                        eprintln!("flag desconocida: {other}");
                        process::exit(2);
                    }
                }
            }
            if let Err(e) = spawn_cmd(Path::new(&dest), BRAIN, force, build) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Some("dish") => {
            let mut steps = DEFAULT_STEPS;
            let mut lambda = DEFAULT_LAMBDA;
            let mut seed = DEFAULT_DISH_SEED;
            let mut delay_ms: u64 = 80;
            let rest: Vec<String> = args.collect();
            let mut it = rest.into_iter();
            while let Some(flag) = it.next() {
                match flag.as_str() {
                    "--steps" => {
                        steps = parse_u32(&next_val(&mut it, "--steps").unwrap_or_else(|e| {
                            eprintln!("error: {e}");
                            process::exit(2);
                        }))
                        .unwrap_or_else(|e| {
                            eprintln!("error: {e}");
                            process::exit(2);
                        });
                    }
                    "--lambda" => {
                        lambda = parse_u32(&next_val(&mut it, "--lambda").unwrap_or_else(|e| {
                            eprintln!("error: {e}");
                            process::exit(2);
                        }))
                        .unwrap_or_else(|e| {
                            eprintln!("error: {e}");
                            process::exit(2);
                        });
                    }
                    "--seed" => {
                        seed = parse_u64(&next_val(&mut it, "--seed").unwrap_or_else(|e| {
                            eprintln!("error: {e}");
                            process::exit(2);
                        }))
                        .unwrap_or_else(|e| {
                            eprintln!("error: {e}");
                            process::exit(2);
                        });
                    }
                    "--delay" => {
                        let v = next_val(&mut it, "--delay").unwrap_or_else(|e| {
                            eprintln!("error: {e}");
                            process::exit(2);
                        });
                        delay_ms = v.parse().unwrap_or_else(|_| {
                            eprintln!("no es un número: {v}");
                            process::exit(2);
                        });
                    }
                    other => {
                        eprintln!("flag desconocida: {other}");
                        process::exit(2);
                    }
                }
            }
            if let Err(e) = dish::run(steps, lambda, seed, delay_ms) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Some(other) => {
            eprintln!("comando desconocido: {other}\n");
            help();
            process::exit(2);
        }
    }
}

fn help() {
    println!(
        "\
demostrante — se reescribe sólo con prueba (generación {GENERATION}, linaje {LINEAGE})
cerebro: {BRAIN}

  demostrante identity              generación, linaje, padre, prueba, sse
  demostrante eval [x]              cerebro vs objetivo
  demostrante prove [expr]          verifica el certificado (o uno contra el cerebro actual)
  demostrante evolve                busca un cerebro mejor
                   --steps N      default {DEFAULT_STEPS}
                   --lambda L     mutantes por paso, default {DEFAULT_LAMBDA}
                   --seed S       rng reproducible
                   --spawn <dir>  hijo con el campeón, si hay prueba
                   --build        compila al hijo
                   --force        pisa un hijo anterior
                   --write        pisa src/main.rs, si hay prueba
  demostrante dish                  anima una célula y un sello
                   --steps N      default {DEFAULT_STEPS}
                   --lambda L     default {DEFAULT_LAMBDA}
                   --seed S       default {DEFAULT_DISH_SEED} (demo fiable)
                   --delay MS     ms entre frames (default 80)
  demostrante spawn <dir>           copia el genoma actual (sin afirmar mejora)
  demostrante genome                imprime las fuentes embebidas

El evaluador está congelado: f(x) = x² + 3x + 5 en x = -5..5.
Un cerebro nuevo no se escribe sin certificado verificable.
El linaje cuenta hijas, no generaciones: 0 → 0.1 → 0.1.1.
No se copia por la red. Un solo hijo por corrida."
    );
}

fn identity() -> Result<(), Box<dyn Error>> {
    let expr = parse_expr(BRAIN)?;
    let err = sse(&expr);
    let buds = read_brotes(Path::new("."));
    println!("demostrante");
    println!("generación  {GENERATION}");
    println!("linaje      {LINEAGE}");
    println!("hijas       {buds}");
    println!("próximo     {}", child_lineage(LINEAGE, buds));
    println!("cerebro     {BRAIN}");
    println!(
        "padre       {}",
        if PARENT_BRAIN.is_empty() {
            "(origen)"
        } else {
            PARENT_BRAIN
        }
    );
    println!(
        "prueba      {}",
        if PROOF.is_empty() { "(ninguna)" } else { PROOF }
    );
    println!("estado      {}", proof_status());
    println!("sse         {err:.4}");
    println!("tamaño      {}", expr.size());
    println!("fitness     {:.4}", score(&expr));
    println!("archivos    {}", GENOME.len());
    println!();
    println!("muestra     x   cerebro   objetivo");
    for x in [0, 1, 2, 3, 5] {
        let xf = x as f64;
        println!(
            "          {x:>2}   {:>7.1}    {:>7.1}",
            expr.eval(xf),
            target(xf)
        );
    }
    if err == 0.0 {
        println!("\nóptimo: el cerebro reproduce el evaluador.");
    }
    Ok(())
}

fn proof_status() -> &'static str {
    if PARENT_BRAIN.is_empty() && PROOF.is_empty() {
        "origen"
    } else if verify_proof(PARENT_BRAIN, BRAIN, PROOF).is_ok() {
        "verificada"
    } else {
        "inválida"
    }
}

fn prove_cmd(cand: Option<&str>) -> Result<(), Box<dyn Error>> {
    match cand {
        None => {
            if PARENT_BRAIN.is_empty() && PROOF.is_empty() {
                println!("origen: no hay teorema que verificar.");
                return Ok(());
            }
            verify_proof(PARENT_BRAIN, BRAIN, PROOF)?;
            println!("padre    {PARENT_BRAIN}");
            println!("cerebro  {BRAIN}");
            println!("prueba   {PROOF}");
            println!("estado   verificada");
        }
        Some(expr) => {
            let proof = make_proof(BRAIN, expr)?;
            verify_proof(BRAIN, expr, &proof)?;
            println!("padre    {BRAIN}");
            println!("cerebro  {expr}");
            println!("prueba   {proof}");
            println!("estado   verificada");
            println!("(nada escrito: usá evolve --spawn / --write para sellar)");
        }
    }
    Ok(())
}

fn improves(parent: &Expr, child: &Expr) -> bool {
    let a = sse(parent);
    let b = sse(child);
    if b < a - 1e-9 {
        true
    } else if (a - b).abs() <= 1e-9 {
        child.size() < parent.size()
    } else {
        false
    }
}

fn format_proof(parent_sse: f64, child_sse: f64) -> String {
    format!("sse:{parent_sse:.4}->{child_sse:.4}")
}

fn parse_proof(s: &str) -> Option<(f64, f64)> {
    let rest = s.strip_prefix("sse:")?;
    let (a, b) = rest.split_once("->")?;
    Some((a.parse().ok()?, b.parse().ok()?))
}

fn make_proof(parent_src: &str, child_src: &str) -> Result<String, Box<dyn Error>> {
    let parent = parse_expr(parent_src)?;
    let child = parse_expr(child_src)?;
    if !improves(&parent, &child) {
        return Err("no hay mejora que demostrar".into());
    }
    Ok(format_proof(sse(&parent), sse(&child)))
}

fn verify_proof(parent_src: &str, child_src: &str, proof: &str) -> Result<(), Box<dyn Error>> {
    if parent_src.is_empty() {
        return Err("origen: no hay padre que demostrar".into());
    }
    if proof.is_empty() {
        return Err("no hay certificado".into());
    }
    let parent = parse_expr(parent_src)?;
    let child = parse_expr(child_src)?;
    if !improves(&parent, &child) {
        return Err("el cerebro no mejora al padre".into());
    }
    let (claim_a, claim_b) = parse_proof(proof).ok_or("certificado ilegible")?;
    let a = sse(&parent);
    let b = sse(&child);
    if (claim_a - a).abs() > 1e-3 || (claim_b - b).abs() > 1e-3 {
        return Err("el certificado no cierra al reevaluar".into());
    }
    Ok(())
}

pub(crate) fn would_prove(champ: &Champion) -> bool {
    parse_expr(BRAIN)
        .ok()
        .map(|parent| improves(&parent, &champ.expr))
        .unwrap_or(false)
}

fn eval_cmd(x_arg: Option<&str>) -> Result<(), Box<dyn Error>> {
    let expr = parse_expr(BRAIN)?;
    match x_arg {
        Some(s) => {
            let x: f64 = s.parse().map_err(|_| format!("no es un número: {s}"))?;
            println!(
                "x={x}  cerebro={}  objetivo={}  err={}",
                expr.eval(x),
                target(x),
                expr.eval(x) - target(x)
            );
        }
        None => {
            println!("  x   cerebro   objetivo     err²");
            for i in -XS..=XS {
                let x = i as f64;
                let y = expr.eval(x);
                let t = target(x);
                let d = y - t;
                println!("{i:>3}  {y:>8.1}  {t:>8.1}  {:>8.1}", d * d);
            }
            println!("sse  {:.4}", sse(&expr));
        }
    }
    Ok(())
}

fn print_genome() {
    for (i, (name, body)) in GENOME.iter().enumerate() {
        if i > 0 {
            println!();
        }
        println!("===== {name} =====");
        print!("{body}");
        if !body.ends_with('\n') {
            println!();
        }
    }
}

struct EvolveOpts {
    steps: u32,
    lambda: u32,
    seed: u64,
    spawn_dir: Option<PathBuf>,
    build: bool,
    force: bool,
    write: bool,
}

fn parse_evolve_opts(args: Vec<String>) -> Result<EvolveOpts, Box<dyn Error>> {
    let mut opts = EvolveOpts {
        steps: DEFAULT_STEPS,
        lambda: DEFAULT_LAMBDA,
        seed: entropy_seed(),
        spawn_dir: None,
        build: false,
        force: false,
        write: false,
    };
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--steps" => opts.steps = parse_u32(&next_val(&mut it, "--steps")?)?,
            "--lambda" => opts.lambda = parse_u32(&next_val(&mut it, "--lambda")?)?,
            "--seed" => opts.seed = parse_u64(&next_val(&mut it, "--seed")?)?,
            "--spawn" => opts.spawn_dir = Some(PathBuf::from(next_val(&mut it, "--spawn")?)),
            "--build" => opts.build = true,
            "--force" => opts.force = true,
            "--write" => opts.write = true,
            other => return Err(format!("flag desconocida: {other}").into()),
        }
    }
    if opts.build && opts.spawn_dir.is_none() {
        return Err("--build pide --spawn <dir>".into());
    }
    Ok(opts)
}

fn next_val(it: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, Box<dyn Error>> {
    it.next()
        .ok_or_else(|| format!("{flag} pide un valor").into())
}

fn parse_u32(s: &str) -> Result<u32, Box<dyn Error>> {
    s.parse().map_err(|_| format!("no es u32: {s}").into())
}

fn parse_u64(s: &str) -> Result<u64, Box<dyn Error>> {
    s.parse().map_err(|_| format!("no es u64: {s}").into())
}

fn entropy_seed() -> u64 {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    t ^ ((process::id() as u64) << 32)
}

fn evolve_cmd(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let opts = parse_evolve_opts(args)?;
    let mut history = Vec::new();
    let mut saw_zero = false;
    let champ = evolve_on(
        opts.steps,
        opts.lambda,
        opts.seed,
        |step, best, improved| {
            history.push((step, best.sse));
            if step == 0 || improved {
                let mark = if best.sse == 0.0 && step > 0 {
                    if saw_zero {
                        "  compactó"
                    } else {
                        "  óptimo"
                    }
                } else if improved {
                    "  *"
                } else {
                    ""
                };
                if best.sse == 0.0 {
                    saw_zero = true;
                }
                println!(
                    "paso {step:>4}  sse {:>10.2}  {}{mark}",
                    best.sse,
                    best.expr.emit()
                );
            }
        },
    )?;
    let improved = champ.sse < sse(&parse_expr(BRAIN)?) - 1e-9;
    let sse0 = history.first().map(|p| p.1).unwrap_or(champ.sse);
    print_sse_chart(&history, sse0);

    println!();
    println!("seed       {}", opts.seed);
    println!("campeón    {}", champ.expr.emit());
    println!("sse        {:.4}", champ.sse);
    println!(
        "{}",
        if champ.sse == 0.0 {
            "resultado   óptimo"
        } else if improved {
            "resultado   mejoró"
        } else {
            "resultado   no mejoró"
        }
    );

    let new_brain = champ.expr.emit();
    match make_proof(BRAIN, &new_brain) {
        Ok(proof) => {
            println!("prueba      {proof}  verificada");
            if opts.write {
                write_local_improvement(&new_brain, BRAIN, &proof)?;
            }
            if let Some(dir) = opts.spawn_dir {
                spawn_improved(&dir, &new_brain, BRAIN, &proof, opts.force, opts.build)?;
            } else if !opts.write {
                println!("(nada escrito: usá --spawn <dir> o --write para sellar)");
            }
        }
        Err(e) => {
            if opts.write || opts.spawn_dir.is_some() {
                return Err(e);
            }
            println!("prueba      (no hay mejora que sellar)");
            println!("(nada escrito: usá --spawn <dir> o --write)");
        }
    }
    Ok(())
}

#[cfg(test)]
fn evolve(steps: u32, lambda: u32, seed: u64) -> Result<Champion, Box<dyn Error>> {
    let mut saw_zero = false;
    evolve_on(steps, lambda, seed, |step, best, improved| {
        if step == 0 || improved {
            let mark = if best.sse == 0.0 && step > 0 {
                if saw_zero {
                    "  compactó"
                } else {
                    "  óptimo"
                }
            } else if improved {
                "  *"
            } else {
                ""
            };
            if best.sse == 0.0 {
                saw_zero = true;
            }
            println!(
                "paso {step:>4}  sse {:>10.2}  {}{mark}",
                best.sse,
                best.expr.emit()
            );
        }
    })
}

pub(crate) fn evolve_on<F>(
    steps: u32,
    lambda: u32,
    seed: u64,
    mut hook: F,
) -> Result<Champion, Box<dyn Error>>
where
    F: FnMut(u32, &Champion, bool),
{
    let mut rng = Rng::new(seed);
    let expr = parse_expr(BRAIN)?;
    let mut best = Champion::from_expr(expr);
    hook(0, &best, false);
    if try_compact(&mut best) {
        hook(0, &best, true);
    }
    for step in 1..=steps {
        let mut winner = best.clone();
        for _ in 0..lambda {
            let m = mutate(&best.expr, &mut rng);
            if m.depth() > MAX_DEPTH {
                continue;
            }
            let cand = Champion::from_expr(m);
            if cand.score < winner.score {
                winner = cand;
            }
        }
        if winner.score < best.score {
            best = winner;
            hook(step, &best, true);
            if try_compact(&mut best) {
                hook(step, &best, true);
            }
        }
    }
    Ok(best)
}

fn try_compact(best: &mut Champion) -> bool {
    if best.sse != 0.0 {
        return false;
    }
    let cand = Champion::from_expr(simplify(&best.expr));
    if cand.sse == 0.0 && cand.score < best.score {
        *best = cand;
        true
    } else {
        false
    }
}

fn write_local_improvement(
    brain: &str,
    parent_brain: &str,
    proof: &str,
) -> Result<(), Box<dyn Error>> {
    verify_proof(parent_brain, brain, proof)?;
    let main_path = Path::new("src/main.rs");
    if !looks_like_demostrante(Path::new(".")) {
        return Err("este directorio no parece un demostrante (corrés desde el crate)".into());
    }
    let src = fs::read_to_string(main_path)?;
    let src = patch_const_str(&src, "BRAIN", brain)?;
    let src = patch_const_str(&src, "PARENT_BRAIN", parent_brain)?;
    let next = patch_const_str(&src, "PROOF", proof)?;
    fs::write(main_path, next)?;
    println!("escribió cerebro y prueba en {}", main_path.display());
    println!("compilá de nuevo para que el binario nazca con ellos");
    Ok(())
}

const BROTES: &str = ".brotes";

pub(crate) fn child_lineage(parent: &str, buds: u32) -> String {
    format!("{parent}.{}", buds + 1)
}

fn spawn_cmd(dest: &Path, brain: &str, force: bool, build: bool) -> Result<(), Box<dyn Error>> {
    let dest = normalize_dest(dest)?;
    let (lineage, record) = plan_birth(Path::new("."), LINEAGE, &dest, force)?;
    spawn_lineage(&dest, brain, PARENT_BRAIN, PROOF, force, build, &lineage)?;
    if record {
        record_birth(Path::new("."))?;
    }
    Ok(())
}

fn spawn_improved(
    dest: &Path,
    brain: &str,
    parent_brain: &str,
    proof: &str,
    force: bool,
    build: bool,
) -> Result<(), Box<dyn Error>> {
    verify_proof(parent_brain, brain, proof)?;
    let dest = normalize_dest(dest)?;
    let (lineage, record) = plan_birth(Path::new("."), LINEAGE, &dest, force)?;
    spawn_lineage(&dest, brain, parent_brain, proof, force, build, &lineage)?;
    if record {
        record_birth(Path::new("."))?;
    }
    Ok(())
}

#[cfg(test)]
fn spawn(dest: &Path, brain: &str, force: bool, build: bool) -> Result<(), Box<dyn Error>> {
    spawn_lineage(
        dest,
        brain,
        PARENT_BRAIN,
        PROOF,
        force,
        build,
        &child_lineage(LINEAGE, 0),
    )
}

fn spawn_lineage(
    dest: &Path,
    brain: &str,
    parent_brain: &str,
    proof: &str,
    force: bool,
    build: bool,
    next_lineage: &str,
) -> Result<(), Box<dyn Error>> {
    let dest = normalize_dest(dest)?;
    assert_safe_dest(&dest)?;
    prepare_dest(&dest, force)?;

    let next_gen = GENERATION + 1;
    let child_main = rewrite_main(
        include_str!("main.rs"),
        next_gen,
        next_lineage,
        brain,
        parent_brain,
        proof,
    )?;

    for (rel, contents) in GENOME {
        let body = if *rel == "src/main.rs" {
            child_main.clone()
        } else {
            (*contents).to_string()
        };
        let path = dest.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, body)?;
        println!("escribió {}", path.display());
    }

    println!(
        "hijo generación {next_gen} linaje {next_lineage} cerebro {brain} → {}",
        dest.display()
    );

    if build {
        let status = Command::new("cargo")
            .arg("build")
            .current_dir(&dest)
            .status()?;
        if !status.success() {
            return Err("cargo build del hijo falló".into());
        }
        println!(
            "hijo compilado: {}/target/debug/demostrante",
            dest.display()
        );
    }
    Ok(())
}

fn rewrite_main(
    src: &str,
    generation: u32,
    lineage: &str,
    brain: &str,
    parent_brain: &str,
    proof: &str,
) -> Result<String, Box<dyn Error>> {
    let src = patch_const_u32(src, "GENERATION", generation)?;
    let src = patch_const_str(&src, "LINEAGE", lineage)?;
    let src = patch_const_str(&src, "BRAIN", brain)?;
    let src = patch_const_str(&src, "PARENT_BRAIN", parent_brain)?;
    patch_const_str(&src, "PROOF", proof)
}

fn patch_const_u32(src: &str, name: &str, new: u32) -> Result<String, Box<dyn Error>> {
    let start_pat = format!("const {name}: u32 = ");
    let start = src
        .find(&start_pat)
        .ok_or_else(|| format!("no encuentro {name} en el genoma"))?;
    let value_start = start + start_pat.len();
    let rel_end = src[value_start..]
        .find(';')
        .ok_or_else(|| format!("const {name} sin cierre"))?;
    let mut out = String::with_capacity(src.len() + 8);
    out.push_str(&src[..value_start]);
    out.push_str(&new.to_string());
    out.push_str(&src[value_start + rel_end..]);
    Ok(out)
}

fn patch_const_str(src: &str, name: &str, new_val: &str) -> Result<String, Box<dyn Error>> {
    if new_val.contains('"') || new_val.contains('\\') {
        return Err("el valor no puede tener comillas ni backslash".into());
    }
    let start_pat = format!("const {name}: &str = \"");
    let start = src
        .find(&start_pat)
        .ok_or_else(|| format!("no encuentro {name} en el genoma"))?;
    let value_start = start + start_pat.len();
    let rel_end = src[value_start..]
        .find('"')
        .ok_or_else(|| format!("const {name} sin cierre"))?;
    let value_end = value_start + rel_end;
    let mut out = String::with_capacity(src.len() + new_val.len());
    out.push_str(&src[..value_start]);
    out.push_str(new_val);
    out.push_str(&src[value_end..]);
    Ok(out)
}

fn read_const_str(dir: &Path, name: &str) -> Option<String> {
    let src = fs::read_to_string(dir.join("src/main.rs")).ok()?;
    let start_pat = format!("const {name}: &str = \"");
    let start = src.find(&start_pat)?;
    let value_start = start + start_pat.len();
    let rel_end = src[value_start..].find('"')?;
    Some(src[value_start..value_start + rel_end].to_string())
}

fn read_brotes(dir: &Path) -> u32 {
    fs::read_to_string(dir.join(BROTES))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn record_birth(parent: &Path) -> Result<(), Box<dyn Error>> {
    if !looks_like_demostrante(parent) {
        return Ok(());
    }
    let n = read_brotes(parent) + 1;
    fs::write(parent.join(BROTES), format!("{n}\n"))?;
    println!(
        "padre  hijas {n}  → próximo linaje {}",
        child_lineage(LINEAGE, n)
    );
    Ok(())
}

fn plan_birth(
    parent: &Path,
    parent_lin: &str,
    dest: &Path,
    force: bool,
) -> Result<(String, bool), Box<dyn Error>> {
    if force && dest.exists() && looks_like_demostrante(dest) {
        if let Some(lin) = read_const_str(dest, "LINEAGE") {
            return Ok((lin, false));
        }
    }
    Ok((child_lineage(parent_lin, read_brotes(parent)), true))
}

pub(crate) fn bin_history(history: &[(u32, f64)], width: usize) -> Vec<Option<f64>> {
    if width == 0 {
        return Vec::new();
    }
    let mut cols = vec![None; width];
    if history.is_empty() {
        return cols;
    }
    let max_step = history.iter().map(|(s, _)| *s).max().unwrap_or(0).max(1);
    for &(step, sse) in history {
        let x = ((step as f64 / max_step as f64) * (width - 1) as f64).round() as usize;
        let x = x.min(width - 1);
        cols[x] = Some(match cols[x] {
            Some(v) => v.min(sse),
            None => sse,
        });
    }
    cols
}

fn print_sse_chart(history: &[(u32, f64)], sse0: f64) {
    const W: usize = 44;
    const H: usize = 10;
    if history.is_empty() {
        return;
    }
    let cols = bin_history(history, W);
    let mut grid = vec![vec![' '; W]; H];
    for (x, v) in cols.iter().enumerate() {
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
        let y = ((1.0 - ratio) * (H - 1) as f64).round() as usize;
        let y = y.min(H - 1);
        grid[y][x] = if sse == 0.0 { '@' } else { '*' };
    }
    println!();
    println!("error (sse)  ·  * búsqueda  @ óptimo");
    for (i, row) in grid.iter().enumerate() {
        let label = if i == 0 {
            format!("{:>8.0}", sse0)
        } else if i + 1 == H {
            format!("{:>8}", 0)
        } else {
            format!("{:>8}", "")
        };
        println!("{} |{}", label, row.iter().collect::<String>());
    }
    let last = history.last().map(|p| p.0).unwrap_or(0);
    println!("         +{}", "-".repeat(W));
    println!("          paso 0{:>w$}", last, w = W.saturating_sub(6));
}

fn normalize_dest(dest: &Path) -> Result<PathBuf, Box<dyn Error>> {
    if dest.as_os_str().is_empty() {
        return Err("directorio vacío".into());
    }
    if dest.is_absolute() {
        Ok(dest.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(dest))
    }
}

fn assert_safe_dest(dest: &Path) -> Result<(), Box<dyn Error>> {
    let cwd = env::current_dir()?;
    if dest == cwd {
        return Err("no voy a sobreescribir el directorio actual".into());
    }
    let home = env::var_os("HOME").map(PathBuf::from);
    let forbidden = [
        PathBuf::from("/"),
        PathBuf::from("/usr"),
        PathBuf::from("/bin"),
        PathBuf::from("/sbin"),
        PathBuf::from("/etc"),
        PathBuf::from("/System"),
        PathBuf::from("/Library"),
        PathBuf::from("/Applications"),
        PathBuf::from("/private"),
    ];
    for p in &forbidden {
        if dest == p {
            return Err(format!("destino prohibido: {}", dest.display()).into());
        }
    }
    if let Some(home) = &home {
        if dest == home {
            return Err("no voy a escribir en $HOME".into());
        }
    }
    Ok(())
}

fn prepare_dest(dest: &Path, force: bool) -> Result<(), Box<dyn Error>> {
    if !dest.exists() {
        fs::create_dir_all(dest)?;
        return Ok(());
    }
    if dest.is_file() {
        return Err(format!("{} es un archivo", dest.display()).into());
    }
    let empty = dest.read_dir()?.next().is_none();
    if empty {
        return Ok(());
    }
    if !force {
        return Err(format!(
            "{} ya existe y no está vacío (usa --force si es un demostrante anterior)",
            dest.display()
        )
        .into());
    }
    if !looks_like_demostrante(dest) {
        return Err(format!("{} no parece un demostrante; no lo borro", dest.display()).into());
    }
    fs::remove_dir_all(dest)?;
    fs::create_dir_all(dest)?;
    Ok(())
}

fn looks_like_demostrante(dir: &Path) -> bool {
    let cargo = fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
    let main = fs::read_to_string(dir.join("src/main.rs")).unwrap_or_default();
    cargo.contains("name = \"demostrante\"")
        && main.contains("const GENERATION:")
        && main.contains("const BRAIN:")
}

pub(crate) fn target(x: f64) -> f64 {
    x * x + 3.0 * x + 5.0
}

pub(crate) fn sse(expr: &Expr) -> f64 {
    let mut s = 0.0;
    for i in -XS..=XS {
        let x = i as f64;
        let y = expr.eval(x);
        if !y.is_finite() {
            return f64::MAX;
        }
        let d = y - target(x);
        s += d * d;
    }
    s
}

fn score(expr: &Expr) -> f64 {
    let e = sse(expr);
    if e == f64::MAX {
        e
    } else {
        e + 0.01 * expr.size() as f64
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expr {
    X,
    Const(i32),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

#[derive(Clone)]
pub(crate) struct Champion {
    pub(crate) expr: Expr,
    pub(crate) sse: f64,
    score: f64,
}

impl Champion {
    fn from_expr(expr: Expr) -> Self {
        let sse = sse(&expr);
        let score = if sse == f64::MAX {
            sse
        } else {
            sse + 0.01 * expr.size() as f64
        };
        Self { expr, sse, score }
    }
}

impl Expr {
    pub(crate) fn eval(&self, x: f64) -> f64 {
        match self {
            Expr::X => x,
            Expr::Const(c) => *c as f64,
            Expr::Add(a, b) => a.eval(x) + b.eval(x),
            Expr::Sub(a, b) => a.eval(x) - b.eval(x),
            Expr::Mul(a, b) => a.eval(x) * b.eval(x),
        }
    }

    fn size(&self) -> usize {
        match self {
            Expr::X | Expr::Const(_) => 1,
            Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) => 1 + a.size() + b.size(),
        }
    }

    fn depth(&self) -> usize {
        match self {
            Expr::X | Expr::Const(_) => 1,
            Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) => 1 + a.depth().max(b.depth()),
        }
    }

    pub(crate) fn emit(&self) -> String {
        match self {
            Expr::X => "x".into(),
            Expr::Const(c) => c.to_string(),
            Expr::Add(a, b) => format!("(+ {} {})", a.emit(), b.emit()),
            Expr::Sub(a, b) => format!("(- {} {})", a.emit(), b.emit()),
            Expr::Mul(a, b) => format!("(* {} {})", a.emit(), b.emit()),
        }
    }
}

fn simplify(expr: &Expr) -> Expr {
    let mut cur = expr.clone();
    for _ in 0..32 {
        let next = simplify_rec(&cur);
        if next == cur {
            return next;
        }
        cur = next;
    }
    cur
}

fn simplify_rec(expr: &Expr) -> Expr {
    match expr {
        Expr::X | Expr::Const(_) => expr.clone(),
        Expr::Mul(a, b) => simplify_mul(simplify_rec(a), simplify_rec(b)),
        Expr::Add(_, _) | Expr::Sub(_, _) => simplify_sum(expr),
    }
}

fn simplify_sum(expr: &Expr) -> Expr {
    let mut parts = Vec::new();
    flatten_sum(expr, 1, &mut parts);
    combine_sum(parts)
}

fn flatten_sum(expr: &Expr, sign: i32, out: &mut Vec<(i32, Expr)>) {
    match expr {
        Expr::Add(a, b) => {
            flatten_sum(a, sign, out);
            flatten_sum(b, sign, out);
        }
        Expr::Sub(a, b) => {
            flatten_sum(a, sign, out);
            flatten_sum(b, -sign, out);
        }
        other => {
            let s = match other {
                Expr::Mul(_, _) => simplify_rec(other),
                _ => other.clone(),
            };
            match s {
                Expr::Add(_, _) | Expr::Sub(_, _) => flatten_sum(&s, sign, out),
                _ => out.push((sign, s)),
            }
        }
    }
}

fn combine_sum(parts: Vec<(i32, Expr)>) -> Expr {
    let mut k: i64 = 0;
    let mut terms: Vec<(Expr, i32)> = Vec::new();
    for (sign, e) in parts {
        match e {
            Expr::Const(c) => {
                k += sign as i64 * c as i64;
            }
            Expr::Mul(a, b) => {
                let (coeff, base) = peel_const_factor(*a, *b);
                let coeff = coeff.saturating_mul(sign);
                push_term(&mut terms, base, coeff);
            }
            other => push_term(&mut terms, other, sign),
        }
    }
    rebuild_sum(k, terms)
}

fn peel_const_factor(a: Expr, b: Expr) -> (i32, Expr) {
    match (a, b) {
        (Expr::Const(c), e) | (e, Expr::Const(c)) => (c, e),
        (a, b) => (1, Expr::Mul(Box::new(a), Box::new(b))),
    }
}

fn push_term(terms: &mut Vec<(Expr, i32)>, e: Expr, c: i32) {
    if c == 0 {
        return;
    }
    if let Some((_, coeff)) = terms.iter_mut().find(|(t, _)| *t == e) {
        *coeff = coeff.saturating_add(c);
    } else {
        terms.push((e, c));
    }
}

fn rebuild_sum(k: i64, terms: Vec<(Expr, i32)>) -> Expr {
    let mut parts: Vec<Expr> = Vec::new();
    for (e, c) in terms {
        if c == 0 {
            continue;
        }
        parts.push(scale(c, e));
    }
    if k != 0 {
        if let Ok(c) = i32::try_from(k) {
            parts.push(Expr::Const(c));
        }
    }
    if parts.is_empty() {
        return Expr::Const(0);
    }
    parts.sort_by_key(|e| matches!(e, Expr::Const(_)));
    let mut acc = parts.remove(0);
    for p in parts {
        acc = Expr::Add(Box::new(acc), Box::new(p));
    }
    acc
}

fn scale(c: i32, e: Expr) -> Expr {
    if c == 1 {
        e
    } else {
        simplify_mul(Expr::Const(c), e)
    }
}

fn simplify_mul(a: Expr, b: Expr) -> Expr {
    match (a, b) {
        (Expr::Const(0), _) | (_, Expr::Const(0)) => Expr::Const(0),
        (Expr::Const(1), e) | (e, Expr::Const(1)) => e,
        (Expr::Const(x), Expr::Const(y)) => match x.checked_mul(y) {
            Some(z) => Expr::Const(z),
            None => Expr::Mul(Box::new(Expr::Const(x)), Box::new(Expr::Const(y))),
        },
        (Expr::Const(c), Expr::Mul(x, y)) | (Expr::Mul(x, y), Expr::Const(c)) => match (*x, *y) {
            (Expr::Const(d), e) | (e, Expr::Const(d)) => match c.checked_mul(d) {
                Some(z) => simplify_mul(Expr::Const(z), e),
                None => Expr::Mul(
                    Box::new(Expr::Const(c)),
                    Box::new(Expr::Mul(Box::new(Expr::Const(d)), Box::new(e))),
                ),
            },
            (x, y) => Expr::Mul(
                Box::new(Expr::Const(c)),
                Box::new(Expr::Mul(Box::new(x), Box::new(y))),
            ),
        },
        (a, b) => Expr::Mul(Box::new(a), Box::new(b)),
    }
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

pub(crate) fn parse_expr(src: &str) -> Result<Expr, Box<dyn Error>> {
    let mut p = Parser {
        s: src.as_bytes(),
        i: 0,
    };
    let e = p.parse()?;
    p.skip();
    if p.i != p.s.len() {
        return Err("sobraron tokens en el cerebro".into());
    }
    Ok(e)
}

impl Parser<'_> {
    fn skip(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }

    fn parse(&mut self) -> Result<Expr, Box<dyn Error>> {
        self.skip();
        if self.i >= self.s.len() {
            return Err("expresión vacía".into());
        }
        match self.s[self.i] {
            b'x' | b'X' => {
                self.i += 1;
                Ok(Expr::X)
            }
            b'(' => {
                self.i += 1;
                self.skip();
                if self.i >= self.s.len() {
                    return Err("falta operador".into());
                }
                let op = self.s[self.i];
                self.i += 1;
                let a = self.parse()?;
                let b = self.parse()?;
                self.skip();
                if self.i >= self.s.len() || self.s[self.i] != b')' {
                    return Err("falta )".into());
                }
                self.i += 1;
                match op {
                    b'+' => Ok(Expr::Add(Box::new(a), Box::new(b))),
                    b'-' => Ok(Expr::Sub(Box::new(a), Box::new(b))),
                    b'*' => Ok(Expr::Mul(Box::new(a), Box::new(b))),
                    _ => Err(format!("operador '{}'", op as char).into()),
                }
            }
            b'-' | b'0'..=b'9' => {
                let start = self.i;
                if self.s[self.i] == b'-' {
                    self.i += 1;
                }
                if self.i >= self.s.len() || !self.s[self.i].is_ascii_digit() {
                    return Err("número inválido".into());
                }
                while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                    self.i += 1;
                }
                let n: i32 = std::str::from_utf8(&self.s[start..self.i])
                    .map_err(|_| "utf8")?
                    .parse()
                    .map_err(|_| "número inválido")?;
                Ok(Expr::Const(n))
            }
            c => Err(format!("token '{}'", c as char).into()),
        }
    }
}

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn frac(&mut self) -> f64 {
        (self.next() >> 11) as f64 / ((1u64 << 53) as f64)
    }

    fn usize(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() as usize) % n
        }
    }

    fn i32(&mut self, lo: i32, hi: i32) -> i32 {
        lo + self.usize((hi - lo + 1) as usize) as i32
    }
}

fn random_leaf(rng: &mut Rng) -> Expr {
    if rng.frac() < 0.5 {
        Expr::X
    } else {
        Expr::Const(rng.i32(-5, 5))
    }
}

fn random_op(rng: &mut Rng) -> u8 {
    match rng.usize(5) {
        0 | 1 => b'+',
        2 | 3 => b'*',
        _ => b'-',
    }
}

fn bin(op: u8, a: Expr, b: Expr) -> Expr {
    match op {
        b'+' => Expr::Add(Box::new(a), Box::new(b)),
        b'-' => Expr::Sub(Box::new(a), Box::new(b)),
        _ => Expr::Mul(Box::new(a), Box::new(b)),
    }
}

fn random_tree(rng: &mut Rng, depth: usize) -> Expr {
    if depth == 0 || rng.frac() < 0.45 {
        random_leaf(rng)
    } else {
        bin(
            random_op(rng),
            random_tree(rng, depth - 1),
            random_tree(rng, depth - 1),
        )
    }
}

fn replace_at<F>(expr: &Expr, at: usize, f: &mut F) -> Expr
where
    F: FnMut(&Expr) -> Expr,
{
    fn rec<F>(expr: &Expr, at: usize, offset: usize, f: &mut F) -> Expr
    where
        F: FnMut(&Expr) -> Expr,
    {
        if offset == at {
            return f(expr);
        }
        match expr {
            Expr::X | Expr::Const(_) => expr.clone(),
            Expr::Add(a, b) => rec_bin(expr, a, b, at, offset, f, b'+'),
            Expr::Sub(a, b) => rec_bin(expr, a, b, at, offset, f, b'-'),
            Expr::Mul(a, b) => rec_bin(expr, a, b, at, offset, f, b'*'),
        }
    }

    fn rec_bin<F>(
        expr: &Expr,
        a: &Expr,
        b: &Expr,
        at: usize,
        offset: usize,
        f: &mut F,
        op: u8,
    ) -> Expr
    where
        F: FnMut(&Expr) -> Expr,
    {
        let left_off = offset + 1;
        let right_off = left_off + a.size();
        if at < right_off {
            bin(op, rec(a, at, left_off, f), b.clone())
        } else if at < right_off + b.size() {
            bin(op, a.clone(), rec(b, at, right_off, f))
        } else {
            expr.clone()
        }
    }

    rec(expr, at, 0, f)
}

fn mutate(expr: &Expr, rng: &mut Rng) -> Expr {
    let n = expr.size();
    let at = rng.usize(n);
    replace_at(expr, at, &mut |node| {
        let r = rng.frac();
        if r < 0.25 {
            random_leaf(rng)
        } else if r < 0.45 {
            match node {
                Expr::Add(a, b) => bin(random_op(rng), *a.clone(), *b.clone()),
                Expr::Sub(a, b) => bin(random_op(rng), *a.clone(), *b.clone()),
                Expr::Mul(a, b) => bin(random_op(rng), *a.clone(), *b.clone()),
                _ => random_leaf(rng),
            }
        } else if r < 0.7 {
            if node.depth() >= MAX_DEPTH {
                node.clone()
            } else if rng.frac() < 0.5 {
                bin(random_op(rng), node.clone(), random_leaf(rng))
            } else {
                bin(random_op(rng), random_leaf(rng), node.clone())
            }
        } else if r < 0.9 {
            match node {
                Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) => {
                    if rng.frac() < 0.5 {
                        *a.clone()
                    } else {
                        *b.clone()
                    }
                }
                _ => random_leaf(rng),
            }
        } else {
            random_tree(rng, 2)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genome_lists_the_project() {
        let names: Vec<_> = GENOME.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"src/main.rs"));
        assert!(names.contains(&"src/dish.rs"));
        assert!(names.contains(&"cell.svg"));
        assert!(names.contains(&"Cargo.toml"));
        assert!(names.contains(&"README.md"));
    }

    #[test]
    fn parse_roundtrip() {
        let src = "(+ (* x x) (+ (* 3 x) 5))";
        let e = parse_expr(src).unwrap();
        assert_eq!(e.emit(), src);
        assert_eq!(e.eval(2.0), 15.0);
    }

    #[test]
    fn target_values() {
        assert_eq!(target(0.0), 5.0);
        assert_eq!(target(1.0), 9.0);
        assert_eq!(target(2.0), 15.0);
    }

    #[test]
    fn sse_of_perfect_brain_is_zero() {
        let e = parse_expr("(+ (* x x) (+ (* 3 x) 5))").unwrap();
        assert_eq!(sse(&e), 0.0);
    }

    #[test]
    fn sse_of_zero_is_bad() {
        let e = parse_expr("0").unwrap();
        assert!(sse(&e) > 1000.0);
    }

    #[test]
    fn lineage_counts_daughters_not_generations() {
        assert_eq!(child_lineage("0", 0), "0.1");
        assert_eq!(child_lineage("0", 1), "0.2");
        assert_eq!(child_lineage("0.1", 0), "0.1.1");
        assert_ne!(child_lineage("0.1", 0), "0.1.2");
    }

    #[test]
    fn rewrite_patches_brain_and_generation() {
        let next = rewrite_main(
            include_str!("main.rs"),
            1,
            "0.1",
            "(+ x 1)",
            "0",
            "sse:1.0000->0.0000",
        )
        .unwrap();
        assert!(next.contains("const GENERATION: u32 = 1;"));
        assert!(next.contains("const LINEAGE: &str = \"0.1\";"));
        assert!(next.contains("const BRAIN: &str = \"(+ x 1)\";"));
        assert!(next.contains("const PARENT_BRAIN: &str = \"0\";"));
        assert!(next.contains("const PROOF: &str = \"sse:1.0000->0.0000\";"));
    }

    #[test]
    fn grandchild_lineage_is_not_generation() {
        let child = rewrite_main(
            include_str!("main.rs"),
            1,
            &child_lineage(LINEAGE, 0),
            BRAIN,
            PARENT_BRAIN,
            PROOF,
        )
        .unwrap();
        let grand = rewrite_main(
            &child,
            2,
            &child_lineage("0.1", 0),
            BRAIN,
            PARENT_BRAIN,
            PROOF,
        )
        .unwrap();
        assert!(grand.contains("const LINEAGE: &str = \"0.1.1\";"));
        assert!(!grand.contains("const LINEAGE: &str = \"0.1.2\";"));
    }

    #[test]
    fn spawn_writes_child_with_brain() {
        let dir = env::temp_dir().join(format!("demostrante-test-{}", process::id()));
        let _ = fs::remove_dir_all(&dir);
        spawn(&dir, "(+ x 1)", false, false).unwrap();
        let child = fs::read_to_string(dir.join("src/main.rs")).unwrap();
        assert!(child.contains("const BRAIN: &str = \"(+ x 1)\";"));
        assert!(child.contains("const GENERATION: u32 = 1;"));
        assert!(child.contains("const LINEAGE: &str = \"0.1\";"));
        assert!(child.contains("const PARENT_BRAIN: &str = \"\";"));
        assert!(child.contains("const PROOF: &str = \"\";"));
        assert!(dir.join("src/dish.rs").exists());
        assert!(dir.join("cell.svg").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn refuses_to_spawn_over_cwd() {
        let cwd = env::current_dir().unwrap();
        let err = spawn(&cwd, BRAIN, true, false).unwrap_err();
        assert!(err.to_string().contains("directorio actual"));
    }

    #[test]
    fn evolve_improves_from_zero() {
        let champ = evolve(80, 25, 1).unwrap();
        let start = sse(&parse_expr(BRAIN).unwrap());
        assert!(champ.sse < start);
    }

    #[test]
    fn seed_7_reaches_zero_and_compacts() {
        let champ = evolve(40, 30, DEFAULT_DISH_SEED).unwrap();
        assert_eq!(champ.sse, 0.0);
        assert!(champ.expr.size() < 11);
    }

    fn emit_simplified(src: &str) -> String {
        simplify(&parse_expr(src).unwrap()).emit()
    }

    #[test]
    fn identities_drop_neutrals() {
        assert_eq!(emit_simplified("(+ x 0)"), "x");
        assert_eq!(emit_simplified("(+ 0 x)"), "x");
        assert_eq!(emit_simplified("(- x 0)"), "x");
        assert_eq!(emit_simplified("(* x 1)"), "x");
        assert_eq!(emit_simplified("(* 1 x)"), "x");
        assert_eq!(emit_simplified("(* x 0)"), "0");
    }

    #[test]
    fn identities_combine_like_terms() {
        assert_eq!(emit_simplified("(+ x x)"), "(* 2 x)");
        assert_eq!(emit_simplified("(- x x)"), "0");
        assert_eq!(emit_simplified("(+ x (+ x x))"), "(* 3 x)");
    }

    #[test]
    fn simplify_preserves_values() {
        let src = "(+ x (+ (+ (+ x x) 5) (* x x)))";
        let e = parse_expr(src).unwrap();
        let s = simplify(&e);
        assert!(s.size() < e.size());
        assert_eq!(sse(&s), 0.0);
        for i in -XS..=XS {
            let x = i as f64;
            assert_eq!(e.eval(x), s.eval(x));
        }
    }

    #[test]
    fn simplify_does_not_grow_a_factored_fit() {
        let e = parse_expr("(- (* x (+ 3 x)) -5)").unwrap();
        let s = simplify(&e);
        assert!(s.size() <= e.size());
        assert_eq!(sse(&s), 0.0);
    }

    #[test]
    fn proof_accepts_better_sse() {
        let p = make_proof("0", "(+ (* x x) (+ (* 3 x) 5))").unwrap();
        verify_proof("0", "(+ (* x x) (+ (* 3 x) 5))", &p).unwrap();
        assert!(p.starts_with("sse:"));
    }

    #[test]
    fn proof_rejects_worse_brain() {
        assert!(make_proof("(+ (* x x) (+ (* 3 x) 5))", "0").is_err());
        assert!(verify_proof("0", "(+ x 1)", "sse:0.0000->0.0000").is_err());
    }

    #[test]
    fn proof_rejects_same_brain() {
        assert!(make_proof("0", "0").is_err());
    }

    #[test]
    fn proof_accepts_smaller_equal_fit() {
        let long = "(+ x (+ (+ (+ x x) 5) (* x x)))";
        let short = "(+ (+ (* 3 x) (* x x)) 5)";
        assert_eq!(sse(&parse_expr(long).unwrap()), 0.0);
        assert_eq!(sse(&parse_expr(short).unwrap()), 0.0);
        let p = make_proof(long, short).unwrap();
        verify_proof(long, short, &p).unwrap();
    }

    #[test]
    fn spawn_improved_requires_proof() {
        let dir = env::temp_dir().join(format!("demostrante-proof-{}", process::id()));
        let _ = fs::remove_dir_all(&dir);
        let brain = "(+ (* x x) (+ (* 3 x) 5))";
        let proof = make_proof("0", brain).unwrap();
        spawn_lineage(
            &dir,
            brain,
            "0",
            &proof,
            false,
            false,
            &child_lineage(LINEAGE, 0),
        )
        .unwrap();
        let child = fs::read_to_string(dir.join("src/main.rs")).unwrap();
        assert!(child.contains(&format!("const BRAIN: &str = \"{brain}\";")));
        assert!(child.contains("const PARENT_BRAIN: &str = \"0\";"));
        assert!(child.contains(&format!("const PROOF: &str = \"{proof}\";")));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn bin_history_drops_toward_zero() {
        let h = [(0, 100.0), (10, 25.0), (20, 0.0)];
        let cols = bin_history(&h, 5);
        assert_eq!(cols[0], Some(100.0));
        assert_eq!(cols[4], Some(0.0));
    }
}
