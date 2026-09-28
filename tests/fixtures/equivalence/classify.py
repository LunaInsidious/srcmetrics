import sys


def classify(values, n):
    """Scores values: positives up, negatives down."""
    score = 0
    for i in range(n):
        if values[i] > 0 and values[i] < 100:
            score += 1
        elif values[i] < 0:
            score -= 1
        else:
            continue
    while score > 10:
        score = score // 2
    return score if score > 0 else 0


# Returns the larger value.
def max2(a, b):
    if a > b:
        return a
    return b
