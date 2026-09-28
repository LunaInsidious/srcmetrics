use std::cmp;

/// Scores values: positives up, negatives down.
fn classify(values: &[i32], n: usize) -> i32 {
    let mut score = 0;
    for i in 0..n {
        if values[i] > 0 && values[i] < 100 {
            score += 1;
        } else if values[i] < 0 {
            score -= 1;
        } else {
            continue;
        }
    }
    while score > 10 {
        score = score / 2;
    }
    if score > 0 {
        return score;
    }
    return 0;
}

// Returns the larger value.
fn max2(a: i32, b: i32) -> i32 {
    if a > b {
        return a;
    }
    return b;
}
