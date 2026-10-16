"""Generate LaTeX fragments for a lab report from the experiment results.

Usage:  conda run -n ec python scripts/report_tables.py lab1

Reads   labN/results/summary.csv, labN/results/checker.csv, labN/results/best/*.txt
Writes  labN/report/generated/results.tex         min / max / avg table
        labN/report/generated/breakdown.tex       average composition of the objective
        labN/report/generated/best_solutions.tex  best tours as node lists
        labN/report/generated/checker.tex         checker verification table
        labN/report/generated/weights.tex         regret weight sweep (if weights.csv exists)

The report \\input{}s these files, so re-running the experiment and this script
updates every number in the report without editing it by hand.
"""

import csv
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
METHOD_ORDER = [
    "random", "nn_end", "nn_anywhere", "greedy_cycle",
    "nn_regret", "nn_weighted", "cycle_regret", "cycle_weighted",
]
METHOD_NAMES = {
    "random": "Random",
    "nn_end": "NN (end only)",
    "nn_anywhere": "NN (any position)",
    "greedy_cycle": "Greedy cycle",
    "nn_regret": "NN 2-regret",
    "nn_weighted": "NN 2-regret + change",
    "cycle_regret": "Cycle 2-regret",
    "cycle_weighted": "Cycle 2-regret + change",
}


# Methods of earlier labs; a table that mixes them with newer ones gets a rule
# between the two groups.
PREVIOUS_LAB = {"random", "nn_end", "nn_anywhere", "greedy_cycle"}


def method_rows(methods: list[str], row) -> list[str]:
    """`row(m)` for every method, with a \\midrule where the earlier labs end."""
    lines = []
    for prev, m in zip([None] + methods, methods):
        if prev in PREVIOUS_LAB and m not in PREVIOUS_LAB:
            lines.append(r"\midrule")
        lines.append(row(m))
    return lines


def method_key(method: str) -> int:
    return METHOD_ORDER.index(method) if method in METHOD_ORDER else len(METHOD_ORDER)


def name(method: str) -> str:
    return METHOD_NAMES.get(method, method.replace("_", r"\_"))


def read_csv(path: Path) -> list[dict[str, str]]:
    with open(path) as f:
        return list(csv.DictReader(f))


def results_table(rows: list[dict[str, str]]) -> str:
    instances = sorted({r["instance"] for r in rows})
    methods = sorted({r["method"] for r in rows}, key=method_key)
    by = {(r["instance"], r["method"]): r for r in rows}
    best_avg = {i: min(float(by[i, m]["avg"]) for m in methods) for i in instances}
    best_min = {i: min(int(by[i, m]["min"]) for m in methods) for i in instances}

    lines = [
        r"\begin{tabular}{l" + "rrr" * len(instances) + "}",
        r"\toprule",
        " & " + " & ".join(rf"\multicolumn{{3}}{{c}}{{{i}}}" for i in instances) + r" \\",
        "".join(rf"\cmidrule(lr){{{2 + 3 * k}-{4 + 3 * k}}}" for k in range(len(instances))),
        "Method" + " & avg & min & max" * len(instances) + r" \\",
        r"\midrule",
    ]
    def row(m: str) -> str:
        cells = []
        for i in instances:
            r = by[i, m]
            avg, lo, hi = float(r["avg"]), int(r["min"]), int(r["max"])
            avg_s = f"{avg:,.0f}".replace(",", r"\,")
            lo_s = f"{lo:,}".replace(",", r"\,")
            hi_s = f"{hi:,}".replace(",", r"\,")
            if avg == best_avg[i]:
                avg_s = rf"\textbf{{{avg_s}}}"
            if lo == best_min[i]:
                lo_s = rf"\textbf{{{lo_s}}}"
            cells += [avg_s, lo_s, hi_s]
        return f"{name(m)} & " + " & ".join(cells) + r" \\"

    lines += method_rows(methods, row)
    lines += [r"\bottomrule", r"\end{tabular}"]
    return "\n".join(lines) + "\n"


def breakdown_table(rows: list[dict[str, str]]) -> str:
    """Median and the average composition of the objective (cycle length, node
    costs, closing edge) per greedy method. Same layout as the results table;
    the average itself is already there. Random is left out, and so are the
    methods of earlier labs when newer ones are present (they are only in the
    results table)."""
    rows = [r for r in rows if r["method"] != "random"]
    if any(r["method"] not in PREVIOUS_LAB for r in rows):
        rows = [r for r in rows if r["method"] not in PREVIOUS_LAB]
    instances = sorted({r["instance"] for r in rows})
    methods = sorted({r["method"] for r in rows}, key=method_key)
    by = {(r["instance"], r["method"]): r for r in rows}
    fmt = lambda v: f"{float(v):,.0f}".replace(",", r"\,")
    cols = ["median", "avg_length", "avg_cost", "avg_closing"]
    lines = [
        r"\begin{tabular}{l" + "rrrr" * len(instances) + "}",
        r"\toprule",
        " & " + " & ".join(rf"\multicolumn{{4}}{{c}}{{{i}}}" for i in instances) + r" \\",
        "".join(rf"\cmidrule(lr){{{2 + 4 * k}-{5 + 4 * k}}}" for k in range(len(instances))),
        "Method" + " & median & length & costs & closing" * len(instances) + r" \\",
        r"\midrule",
    ]
    lines += method_rows(
        methods,
        lambda m: f"{name(m)} & " + " & ".join(fmt(by[i, m][c]) for i in instances for c in cols) + r" \\",
    )
    lines += [r"\bottomrule", r"\end{tabular}"]
    return "\n".join(lines) + "\n"


def best_solutions(best_dir: Path) -> str:
    files = sorted(best_dir.glob("*.txt"), key=lambda p: (p.stem.split("_", 1)[0], method_key(p.stem.split("_", 1)[1])))
    out = []
    for path in files:
        instance, method = path.stem.split("_", 1)
        nodes = ", ".join(path.read_text().split())
        out.append(rf"\paragraph{{{instance}, {name(method)}}}" + "\n" + rf"{{\footnotesize\ttfamily\raggedright {nodes}\par}}")
    return "\n\n".join(out) + "\n"


def checker_table(rows: list[dict[str, str]]) -> str:
    instances = sorted({r["instance"] for r in rows})
    methods = sorted({r["method"] for r in rows}, key=method_key)
    by = {(r["instance"], r["method"]): r for r in rows}
    lines = [
        r"\begin{tabular}{l" + "rr" * len(instances) + "}",
        r"\toprule",
        " & " + " & ".join(rf"\multicolumn{{2}}{{c}}{{{i}}}" for i in instances) + r" \\",
        "".join(rf"\cmidrule(lr){{{2 + 2 * k}-{3 + 2 * k}}}" for k in range(len(instances))),
        "Method" + " & reported & checker" * len(instances) + r" \\",
        r"\midrule",
    ]
    def row(m: str) -> str:
        cells = []
        for i in instances:
            r = by[i, m]
            checker = r["checker"] if r["match"] == "True" else rf"\textbf{{{r['checker']} (mismatch)}}"
            cells += [r["reported"], checker]
        return f"{name(m)} & " + " & ".join(cells) + r" \\"

    lines += method_rows(methods, row)
    lines += [r"\bottomrule", r"\end{tabular}"]
    return "\n".join(lines) + "\n"


def weights_table(rows: list[dict[str, str]]) -> str:
    """Average and minimum objective for every weight of the regret, per base
    heuristic and instance; the best average in each column pair is in bold."""
    instances = sorted({r["instance"] for r in rows})
    bases = ["nn", "cycle"]
    lambdas = sorted({r["lambda"] for r in rows}, key=float)
    by = {(r["instance"], r["base"], r["lambda"]): r for r in rows}
    groups = [(i, b) for i in instances for b in bases]
    best_avg = {g: min(float(by[*g, l]["avg"]) for l in lambdas) for g in groups}
    fmt = lambda v: f"{float(v):,.0f}".replace(",", r"\,")
    base_name = {"nn": "NN", "cycle": "cycle"}
    lines = [
        r"\begin{tabular}{r" + "rr" * len(groups) + "}",
        r"\toprule",
        " & " + " & ".join(rf"\multicolumn{{4}}{{c}}{{{i}}}" for i in instances) + r" \\",
        "".join(rf"\cmidrule(lr){{{2 + 4 * k}-{5 + 4 * k}}}" for k in range(len(instances))),
        " & " + " & ".join(rf"\multicolumn{{2}}{{c}}{{{base_name[b]}}}" for _, b in groups) + r" \\",
        "".join(rf"\cmidrule(lr){{{2 + 2 * k}-{3 + 2 * k}}}" for k in range(len(groups))),
        r"$\lambda$" + " & avg & min" * len(groups) + r" \\",
        r"\midrule",
    ]
    for l in lambdas:
        cells = []
        for g in groups:
            r = by[*g, l]
            avg_s = fmt(r["avg"])
            if float(r["avg"]) == best_avg[g]:
                avg_s = rf"\textbf{{{avg_s}}}"
            cells += [avg_s, fmt(r["min"])]
        lines.append(f"{l} & " + " & ".join(cells) + r" \\")
    lines += [r"\bottomrule", r"\end{tabular}"]
    return "\n".join(lines) + "\n"


def main(lab: str):
    results = ROOT / lab / "results"
    out = ROOT / lab / "report" / "generated"
    out.mkdir(parents=True, exist_ok=True)

    summary = read_csv(results / "summary.csv")
    (out / "results.tex").write_text(results_table(summary))
    (out / "breakdown.tex").write_text(breakdown_table(summary))
    (out / "best_solutions.tex").write_text(best_solutions(results / "best"))
    weights = results / "weights.csv"
    if weights.exists():
        (out / "weights.tex").write_text(weights_table(read_csv(weights)))
    checker = results / "checker.csv"
    if checker.exists():
        (out / "checker.tex").write_text(checker_table(read_csv(checker)))
    else:
        print("warning: no checker.csv, run scripts/check_solutions.py first")
    print(f"LaTeX fragments written to {out}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
