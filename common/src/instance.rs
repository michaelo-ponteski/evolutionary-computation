use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// `labs/problem/`, resolved at compile time relative to this crate, so every
/// lab finds the instances regardless of the working directory.
pub fn problem_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../problem")
}

/// Problem instance as seen by the optimisation methods.
///
/// Coordinates are deliberately NOT stored here: the methods may only touch the
/// distance matrix and node costs, so they also work on matrix-only instances.
/// Plotting reads coordinates straight from the CSV instead.
pub struct Instance {
    pub name: String,
    pub n: usize,
    pub costs: Vec<i32>,
    /// Row-major n*n matrix: dist[i * n + j].
    dist: Vec<i32>,
}

impl Instance {
    /// Reads a `x;y;cost` CSV and precomputes the distance matrix.
    pub fn from_csv(path: &Path) -> Result<Self, Box<dyn Error>> {
        let text = fs::read_to_string(path)?;

        let mut xs = Vec::new();
        let mut ys = Vec::new();
        let mut costs = Vec::new();
        for (line_no, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let fields: Vec<i32> = line
                .split(';')
                .map(|f| f.trim().parse::<i32>())
                .collect::<Result<_, _>>()
                .map_err(|e| format!("{}:{}: {e}", path.display(), line_no + 1))?;
            let [x, y, c] = fields[..] else {
                return Err(
                    format!("{}:{}: expected 3 fields", path.display(), line_no + 1).into(),
                );
            };
            xs.push(x);
            ys.push(y);
            costs.push(c);
        }

        let n = costs.len();
        let mut dist = vec![0; n * n];
        for i in 0..n {
            for j in 0..n {
                let dx = (xs[i] - xs[j]) as f64;
                let dy = (ys[i] - ys[j]) as f64;
                // f64::round rounds half away from zero; distances are >= 0,
                // so this is the same as the checker's INT(x + 0.5).
                dist[i * n + j] = (dx * dx + dy * dy).sqrt().round() as i32;
            }
        }

        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        Ok(Self {
            name,
            n,
            costs,
            dist,
        })
    }

    /// Loads an instance file (e.g. `"TSPA.csv"`) from [`problem_dir`].
    pub fn load(file: &str) -> Result<Self, Box<dyn Error>> {
        Self::from_csv(&problem_dir().join(file))
    }

    #[inline]
    pub fn d(&self, i: usize, j: usize) -> i32 {
        self.dist[i * self.n + j]
    }

    /// Number of nodes a solution must visit: 50%, rounded up.
    pub fn select_count(&self) -> usize {
        self.n.div_ceil(2)
    }
}
