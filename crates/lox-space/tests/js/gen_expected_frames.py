from pathlib import Path
import json
import lox_space as lox
import spiceypy as spice

FRAMES = ["IAU_EARTH", "IAU_MOON"]


## This generated the expected data we verify in test_frames.js
## To regenerate, run this from within crates/lox_space
##   uv run python tests/js/gen_expected_frames.py
def main():
    # ${PROJECT_ROOT}/data/spice
    data_dir = Path(__file__).parent.parent.parent.parent.parent / "data" / "spice"
    lsk = data_dir / "naif0012.tls"
    pck = data_dir / "pck00011.tpc"
    spice.furnsh([str(lsk), str(pck)])

    t = lox.Time("TDB", 2000, 1, 1)
    et = t.julian_date(epoch="j2000", unit="seconds")
    r0 = (6068.27927, -1692.84394, -2516.61918)
    v0 = (-0.660415582, 5.495938726, -5.303093233)

    out = {}
    for frame in FRAMES:
        ms = spice.sxform("J2000", frame, et)
        s1 = ms @ (*r0, *v0)
        out[frame] = {
            "r": s1[0:3].tolist(),
            "v": s1[3:].tolist(),
        }

    out_path = Path(__file__).parent / "data" / "iau_frames_expected_frames.json"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(out, indent=2))
    print(f"Wrote {out_path}")

if __name__ == "__main__":
    main()
