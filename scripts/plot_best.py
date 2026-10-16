"""Plot the best solutions of a lab.

Usage:  conda run -n ec python scripts/plot_best.py lab1

Reads   labN/results/best/<INSTANCE>_<method>.txt  (one node index per line)
        problem/<INSTANCE>.csv                      (x;y;cost)
Writes  labN/plots/<INSTANCE>_<method>.png          one plot per solution
        labN/plots/<INSTANCE>.png                   all methods side by side

The objective shown in each title is recomputed here independently of the Rust
code, with the solution checker's formula INT(SQRT(...) + 0.5), so the plots
double as a cross-check of the reported values.
"""

import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
import numpy as np

ROOT = Path(__file__).resolve().parent.parent
METHOD_ORDER = [
    "random", "nn_end", "nn_anywhere", "greedy_cycle",
    "nn_regret", "nn_weighted", "cycle_regret", "cycle_weighted",
]


def load_instance(name: str) -> np.ndarray:
    """Rows of (x, y, cost)."""
    return np.loadtxt(ROOT / "problem" / f"{name}.csv", delimiter=";", dtype=np.int64)


def objective(nodes: np.ndarray, tour: list[int]) -> int:
    xy = nodes[tour, :2]
    diff = xy - np.roll(xy, -1, axis=0)  # edge i -> i+1, including last -> first
    length = np.floor(np.sqrt((diff**2).sum(axis=1)) + 0.5).astype(np.int64).sum()
    return int(length + nodes[tour, 2].sum())


def draw(ax: plt.Axes, nodes: np.ndarray, tour: list[int], title: str):
    x, y, cost = nodes[:, 0], nodes[:, 1], nodes[:, 2]
    selected = np.zeros(len(nodes), dtype=bool)
    selected[tour] = True
    # Cost is shown twice: by colour and by marker size.
    size = 15 + 85 * (cost - cost.min()) / max(cost.max() - cost.min(), 1)
    norm = plt.Normalize(cost.min(), cost.max())

    cycle = tour + tour[:1]
    ax.plot(x[cycle[:-1]], y[cycle[:-1]], color="0.25", lw=0.9, zorder=1)
    ax.plot(x[cycle[-2:]], y[cycle[-2:]], color="tab:red", lw=1.4, ls="--", zorder=1)
    ax.scatter(x[~selected], y[~selected], s=size[~selected], c=cost[~selected],
               cmap="viridis", norm=norm, alpha=0.25, edgecolors="none", zorder=2)
    sc = ax.scatter(x[selected], y[selected], s=size[selected], c=cost[selected],
                    cmap="viridis", norm=norm, edgecolors="black", linewidths=0.4, zorder=3)
    ax.set_title(title, fontsize=10)
    ax.set_aspect("equal")
    ax.tick_params(labelsize=7)
    return sc


def legend(fig: plt.Figure):
    """Explains the encoding once per figure, below the plots."""
    handles = [
        Line2D([], [], color="0.25", lw=0.9, label="cycle edge"),
        Line2D([], [], color="tab:red", lw=1.4, ls="--", label="closing edge (last -> first)"),
        Line2D([], [], ls="", marker="o", color="0.5", markeredgecolor="black",
               label="selected node (colour/size = cost)"),
        Line2D([], [], ls="", marker="o", color="0.5", alpha=0.25, label="unselected node"),
    ]
    fig.legend(handles=handles, loc="outside lower center", ncols=4, fontsize=8, frameon=False)


def main(lab: str):
    best_dir = ROOT / lab / "results" / "best"
    out_dir = ROOT / lab / "plots"
    out_dir.mkdir(exist_ok=True)

    solutions: dict[str, dict[str, list[int]]] = {}
    for f in sorted(best_dir.glob("*.txt")):
        instance, method = f.stem.split("_", 1)
        solutions.setdefault(instance, {})[method] = [int(v) for v in f.read_text().split()]

    for instance, by_method in solutions.items():
        nodes = load_instance(instance)
        methods = sorted(by_method, key=lambda m: METHOD_ORDER.index(m) if m in METHOD_ORDER else 99)

        cols = 2
        rows = -(-len(methods) // cols)
        grid, axes = plt.subplots(
            rows, cols, figsize=(6.5 * cols, 3.6 * rows), squeeze=False, layout="constrained"
        )
        for ax, method in zip(axes.flat, methods):
            tour = by_method[method]
            title = f"{instance}: {method}  (objective {objective(nodes, tour)})"
            print(title)

            single, single_ax = plt.subplots(figsize=(10, 5.6), layout="constrained")
            sc = draw(single_ax, nodes, tour, title)
            single.colorbar(sc, ax=single_ax, label="node cost")
            legend(single)
            single.savefig(out_dir / f"{instance}_{method}.png", dpi=150)
            plt.close(single)

            sc = draw(ax, nodes, tour, title)
        for ax in axes.flat[len(methods):]:
            ax.axis("off")
        grid.colorbar(sc, ax=axes, label="node cost", shrink=0.8)
        legend(grid)
        grid.savefig(out_dir / f"{instance}.png", dpi=130, bbox_inches="tight")
        plt.close(grid)

    print(f"plots written to {out_dir}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
