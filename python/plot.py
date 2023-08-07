import matplotlib.pyplot as plt
import numpy as np
import sys

from typing import NamedTuple, Optional, List, TypeVar


class DataPoint(NamedTuple):
    arc_or_clone: str
    num_strings: int
    string_len: int
    threads: int
    ops: int
    seconds: float
    ops_per_sec: float

    @classmethod
    # header is:
    # arc_or_clone num_strings string_len threads operations seconds ops_per_sec
    def from_str(self, input: str):  # -> Optional[DataPoint]
        fields = input.split()
        if fields[0] == "arc_or_clone":
            return None
        # try:
        return DataPoint(
            "Arc" if fields[0] == "A" else "Clone",
            int(fields[1]),
            int(fields[2]),
            int(fields[3]),
            int(fields[4]),
            float(fields[5]),
            float(fields[6]))
        # except:
        #    return None


def parse_input() -> List[DataPoint]:
    data: List[DataPoint] = []

    for line in sys.stdin:
        point = DataPoint.from_str(line)
        if point:
            data.append(point)
    return data


def main():
    all_points = parse_input()

    # get set of points.num_strings values
    num_strings_vals = set([p.num_strings for p in all_points])

    # draw graph from points

    # plot ops_per_sec versus string_len
    # for each arc_or_clone
    for num_strings in num_strings_vals:
        fig, ax = plt.subplots()
        points = [p for p in all_points if p.num_strings == num_strings]
        for arc_or_clone in ["Arc", "Clone"]:
            x = np.array(
                [p.string_len for p in points if p.arc_or_clone == arc_or_clone])
            y = np.array(
                [p.ops_per_sec for p in points if p.arc_or_clone == arc_or_clone])
            ax.plot(x, y, label=f'{arc_or_clone} {num_strings} strings')
        ax.set_xticks([1, 16, 64, 512, 1024])
        ax.set_xlabel("String length")
        ax.set_ylabel("Ops per second")
        ax.set_title(f"Performance with set of {num_strings} strings")
        ax.legend()
        # save image
        plt.savefig(f"arcbench-{num_strings}.png")


if __name__ == "__main__":
    main()
