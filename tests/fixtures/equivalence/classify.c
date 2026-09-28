#include <stdio.h>

/* Scores values: positives up, negatives down. */
int classify(int *values, int n) {
    int score = 0;
    for (int i = 0; i < n; i++) {
        if (values[i] > 0 && values[i] < 100) {
            score += 1;
        } else if (values[i] < 0) {
            score -= 1;
        } else {
            continue;
        }
    }
    while (score > 10) {
        score = score / 2;
    }
    return score > 0 ? score : 0;
}

// Returns the larger value.
int max2(int a, int b) {
    if (a > b) {
        return a;
    }
    return b;
}
