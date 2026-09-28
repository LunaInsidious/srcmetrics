package classify

import "fmt"

// Scores values: positives up, negatives down.
func classify(values []int, n int) int {
	score := 0
	for i := 0; i < n; i++ {
		if values[i] > 0 && values[i] < 100 {
			score += 1
		} else if values[i] < 0 {
			score -= 1
		} else {
			continue
		}
	}
	for score > 10 {
		score = score / 2
	}
	if score > 0 {
		return score
	}
	return 0
}

// Returns the larger value.
func max2(a int, b int) int {
	if a > b {
		return a
	}
	return b
}
