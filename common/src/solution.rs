use crate::instance::Instance;

/// A solution is the visiting order of the selected nodes; the cycle closes
/// implicitly from the last node back to the first.
pub type Tour = Vec<usize>;

/// Cycle length + sum of costs of the visited nodes.
pub fn objective(inst: &Instance, tour: &[usize]) -> i32 {
    let m = tour.len();
    let length: i32 = (0..m).map(|i| inst.d(tour[i], tour[(i + 1) % m])).sum();
    let cost: i32 = tour.iter().map(|&v| inst.costs[v]).sum();
    length + cost
}

/// Checks feasibility: right number of nodes, all in range, no repeats.
pub fn validate(inst: &Instance, tour: &[usize]) -> Result<(), String> {
    if tour.len() != inst.select_count() {
        return Err(format!(
            "expected {} nodes, got {}",
            inst.select_count(),
            tour.len()
        ));
    }
    let mut seen = vec![false; inst.n];
    for &v in tour {
        if v >= inst.n {
            return Err(format!("node {v} out of range"));
        }
        if seen[v] {
            return Err(format!("node {v} visited twice"));
        }
        seen[v] = true;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn load(name: &str) -> Instance {
        Instance::load(name).unwrap()
    }

    // Example node lists shipped in "Solution checker.xlsx" with the value the
    // spreadsheet computes. They contain repeated nodes (they are not feasible),
    // but they pin down our distance rounding and cycle closing to the checker's.
    #[test]
    fn matches_checker_tspa() {
        let tour = [
            140, 108, 69, 18, 22, 193, 41, 139, 42, 160, 177, 10, 190, 4, 112, 184, 43, 116, 65,
            123, 127, 162, 151, 133, 135, 154, 180, 53, 100, 26, 86, 75, 120, 44, 25, 16, 171, 175,
            113, 56, 31, 78, 145, 196, 81, 90, 165, 40, 185, 106, 178, 49, 14, 144, 62, 9, 148,
            167, 52, 55, 57, 92, 129, 82, 2, 101, 1, 97, 152, 124, 94, 63, 80, 176, 137, 51, 118,
            59, 115, 46, 0, 89, 183, 143, 117, 93, 101, 99, 135, 51, 112, 66, 6, 156, 98, 190, 72,
            94, 12, 73,
        ];
        assert_eq!(objective(&load("TSPA.csv"), &tour), 253566);
    }

    #[test]
    fn matches_checker_tspb() {
        let tour = [
            90, 51, 121, 131, 122, 133, 107, 40, 63, 135, 38, 27, 16, 1, 156, 198, 117, 193, 31,
            54, 73, 136, 190, 80, 45, 175, 78, 5, 177, 36, 61, 91, 141, 77, 81, 153, 187, 163, 89,
            127, 137, 114, 103, 113, 176, 194, 166, 86, 185, 95, 130, 99, 179, 66, 94, 47, 148, 60,
            20, 28, 149, 4, 140, 183, 152, 170, 34, 55, 18, 62, 124, 106, 143, 35, 109, 0, 29, 160,
            33, 144, 111, 82, 21, 8, 104, 138, 11, 139, 168, 195, 13, 145, 15, 3, 70, 132, 169,
            188, 6, 147,
        ];
        assert_eq!(objective(&load("TSPB.csv"), &tour), 200088);
    }

    #[test]
    fn validate_checks_feasibility() {
        let inst = load("TSPA.csv");
        let dup: Vec<usize> = (0..99).chain([0]).collect();
        assert!(validate(&inst, &dup).is_err());
        let ok: Tour = (0..100).collect();
        assert!(validate(&inst, &ok).is_ok());
    }
}
