import random


NUM_ATOMS = 1_000_000
COORD_RANGE = (-300, 300)


def generate_coord() -> str:
    coord = random.uniform(*COORD_RANGE)
    formatted = f"{coord:.3f}"
    padding = " " * (8 - len(formatted))
    return padding + formatted


def generate_line():
    x, y, z = [generate_coord() for _ in range(3)]
    element = random.sample(list("CNOHSP"), 1).pop()
    line = f"ATOM {' ' * 24} {x}{y}{z} {' ' * 22}{element}  "
    return line


def generate_pdb_file(filename: str = "test.pdb"):
    with open(filename, "w") as f:
        for _ in range(NUM_ATOMS):
            f.write(generate_line() + "\n")


if __name__ == "__main__":
    generate_pdb_file()
