#!/usr/bin/env python3
"""Write Malay search words for every symbol, a batch at a time, when usage allows.

Reads ~/.claude/usage.json (written by the status line). Batches go to out/,
so a stopped run picks up where it left off. Run from the repo root:
  scripts/malay-tags/run.py <media dir> <work dir> [max batches]
"""
import concurrent.futures as cf, datetime as dt, json, os, pathlib, subprocess, sys, threading, time

MEDIA, WORK = pathlib.Path(sys.argv[1]).resolve(), pathlib.Path(sys.argv[2]).resolve()
MAX = int(sys.argv[3]) if len(sys.argv) > 3 else 10**9
HERE = pathlib.Path(__file__).parent
OUT = HERE / "out"
SIZE, WORKERS, COMMIT_EVERY = 75, 4, 8
FIVE_HOUR_MAX = 50
WEEK_RESET = dt.datetime(2026, 11, 7, 2, 0, tzinfo=dt.timezone.utc)
USAGE = pathlib.Path.home() / ".claude/usage.json"
PROMPT = (HERE / "prompt.txt").read_text()
MAX_PER_5H = 28
lock, done_since_commit, starts = threading.Lock(), [0], []


def log(msg):
    print(dt.datetime.now().strftime("%H:%M:%S"), msg, flush=True)


def week_max():
    # Use up the week before it resets, then back to the usual limit.
    return 90 if dt.datetime.now(dt.timezone.utc) < WEEK_RESET else 70


def wait_for_room():
    # usage.json can lag while sessions are idle, so also cap batches per 5 hours.
    while True:
        with lock:
            now = time.time()
            starts[:] = [t for t in starts if now - t < 5 * 3600]
            over = len(starts) >= MAX_PER_5H
        if over:
            log(f"waiting: {MAX_PER_5H} batches in the last 5 hours")
            time.sleep(300)
            continue
        try:
            u = json.loads(USAGE.read_text())
            age = time.time() - u["saved_at"]
            h, w = u["five_hour"]["used_percentage"], u["seven_day"]["used_percentage"]
            if age < 900 and h < FIVE_HOUR_MAX and w < week_max():
                with lock:
                    starts.append(time.time())
                return
            log(f"waiting: 5h {h}% week {w}% (max {week_max()}) data {int(age)}s old")
        except Exception as e:
            log(f"waiting: can't read usage ({e})")
            time.sleep(10)
            continue
        time.sleep(300)


def commit():
    subprocess.run("git add scripts/malay-tags && git commit -q -m 'Malay tags: more batches' && git push -q origin malay-tags",
                   shell=True, capture_output=True)


def run_batch(n, items):
    final = OUT / f"batch-{n:04d}.json"
    if final.exists():
        return
    batch, out = WORK / f"batch-{n:04d}-in.json", WORK / f"batch-{n:04d}-out.json"
    want = [str(MEDIA / s["file"]) for s in items]
    batch.write_text(json.dumps([{"id": i, "name": s["name"], "path": p}
                                 for i, (s, p) in enumerate(zip(items, want))]))
    for attempt in (1, 2):
        wait_for_room()
        log(f"batch {n} attempt {attempt}")
        result = None
        try:
            result = subprocess.run(["claude", "-p", PROMPT.format(batch=batch, out=out), "--model", "sonnet",
                          "--allowedTools", "Read", "Write", "--add-dir", str(MEDIA), str(WORK),
                          "--permission-mode", "acceptEdits"],
                         cwd=WORK, capture_output=True, text=True, timeout=1800, stdin=subprocess.DEVNULL)
        except subprocess.TimeoutExpired:
            log(f"batch {n}: timed out")
            continue
        try:
            raw = json.loads(out.read_text())
            tags = {p: raw.get(str(i)) for i, p in enumerate(want)}
            missing = [p for p in want if not tags.get(p)]
            if len(missing) <= 2:
                prefix = str(MEDIA) + "/"
                final.write_text(json.dumps({p.removeprefix(prefix): tags[p] for p in want if tags.get(p)},
                                            ensure_ascii=False, indent=0))
                log(f"batch {n} done, {len(want) - len(missing)}/{len(want)}")
                with lock:
                    done_since_commit[0] += 1
                    if done_since_commit[0] >= COMMIT_EVERY:
                        done_since_commit[0] = 0
                        commit()
                return
            log(f"batch {n}: {len(missing)} missing")
        except Exception as e:
            log(f"batch {n}: bad output ({e}): {result.stdout[-300:]!r} {result.stderr[-200:]!r}")
    log(f"batch {n} FAILED")


def main():
    WORK.mkdir(parents=True, exist_ok=True)
    pilot = json.loads((OUT / "pilot.json").read_text())
    order = {"custom": 0, "tawasol-symbols": 1, "mulberry-symbols": 2, "arasaac-symbols": 3}
    todo = [s for s in json.loads((HERE / "symbols-list.json").read_text()) if s["file"] not in pilot]
    todo.sort(key=lambda s: (order.get(s["file"].split("/")[0], 9), s["file"]))
    batches = [todo[i:i + SIZE] for i in range(0, len(todo), SIZE)]
    pending = [(n, b) for n, b in enumerate(batches) if not (OUT / f"batch-{n:04d}.json").exists()][:MAX]
    log(f"{len(batches)} batches, {len(pending)} to run")
    with cf.ThreadPoolExecutor(WORKERS) as ex:
        list(ex.map(lambda a: run_batch(*a), pending))
    commit()
    log("finished")


main()
