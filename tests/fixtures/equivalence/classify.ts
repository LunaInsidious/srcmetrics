import { stdout } from "process";

/** Scores values: positives up, negatives down. */
function classify(values: number[], n: number): number {
    let score = 0;
    for (let i = 0; i < n; i++) {
        if (values[i] > 0 && values[i] < 100) {
            score += 1;
        } else if (values[i] < 0) {
            score -= 1;
        } else {
            continue;
        }
    }
    while (score > 10) {
        score = Math.floor(score / 2);
    }
    return score > 0 ? score : 0;
}

// Returns the larger value.
function max2(a: number, b: number): number {
    if (a > b) {
        return a;
    }
    return b;
}
