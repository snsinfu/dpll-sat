# dpll-sat

[![Build Status][build-badge]][build-url]

[build-badge]: https://github.com/snsinfu/dpll-sat/workflows/test/badge.svg
[build-url]: https://github.com/snsinfu/dpll-sat/actions?query=workflow%3Atest

**dpll-sat** is a SAT solver implementing the classic [DPLL algorithm][dpll]. I
wrote this program for learning purposes and also for comparing the performance
of a naive DPLL solver and modern solvers.

- [Build](#build)
- [Usage](#usage)
- [Implementation notes](#implementation-notes)
- [Benchmarks](#benchmarks)
- [References](#references)

[dpll]: https://en.wikipedia.org/wiki/DPLL_algorithm

## Build

```console
$ git clone https://github.com/snsinfu/dpll-sat
$ cd dpll-sat
$ cargo build --release
$ cp target/release/dpll-sat ~/bin/
```

## Usage

**dpll-sat** command reads [a simplified DIMACS CNF][format] from stdin. It
prints "sat" followed by an assignment and exits with exit code 0 if the formula
is satisfiable. Otherwise, it prints "unsat" and exits with exit code 1.

```console
$ dpll-sat < examples/qg3-08.cnf
sat
1 2 3 4 5 6 7 8 -9 -10 -11 -12 -13 -14 -15 -16 -17 ...
```

The second line shows an assignment. A positive number `i` means that the i-th
variable is true. A negative number `-i` means that the i-th variable is false.
The output format is essentially the same as that of [z3][z3].

[format]: http://www.satcompetition.org/2011/format-benchmarks2011.html
[z3]: https://github.com/Z3Prover/z3

## Implementation notes

The algorithm is implemented in the following way in pseudocode:

```javascript
function DPLL(formula, vars) {
    // Simplify formula by determining variables in unit clauses.
    formula = copy(formula);
    unit_propagate(formula, vars);

    // Stopping conditions.
    if (formula is empty) {
        return "SAT";
    }
    if (empty clause in formula) {
        return "UNSAT";
    }

    // Choose a branching literal and recurse.
    v = choose_literal(formula);

    return DPLL(formula + [v], vars) || DPLL(formula + [not(v)], vars);
}
```

- Pure literal elimination is not implemented due to the complexity of keeping
  track of the polarity of every literal in a formula.
- The branching literal is chosen to be the most-used variable in a formula.
  The literal is eliminated in the next recursion, so this strategy eagerly
  reduces the size of the formula.
- Every recursion creates a new copy of a formula. This is inefficient. The copy
  is necessary because the algorithm eliminates some clauses and literals in a
  formula and later revert it for backtracking. But, most clauses are untouched.
  There would be a clever data structure that can reduce the number of copies.
  Maybe deque?
- A formula is represented as a vector-of-vectors. It's horribly inefficient
  since a formula tends to consist of many small clauses, making memory access
  extremely scattered. It would be much better to use a flat vector with
  sentinel values.

## Benchmarks

Run time for some benchmark instances found on [the satlib site][satlib] and a
[benchmark page][bench]. The run time is the total "USER" time spent from a
program startup to termination. Benchmarks ran on Fedora 33 Linux with AMD Ryzen
3600 CPU @ 4.2GHz (boost).

| Instance                                  | #var | #clause | dpll   | z3     | cadical |
|-------------------------------------------|------|---------|--------|--------|---------|
| Random 3-SAT CBS_k3_n100_m403_b10_0.cnf   | 100  | 403     | 0.03s  |  0.01s | 0.00s   |
| blocksworld huge.cnf                      | 459  | 7054    | 0.15s  |  0.00s | 0.00s   |
| SAT-02 glassy-sat-sel_N210_n.shuffled.cnf | 210  | 980     | 215min |  0.85s | 0.31s   |
| SAT-02 homer10.shuffled.cnf               | 360  | 3460    | > 1day | 12.77s | 4.22s   |

Z3 does pretty good job considering [CaDiCaL][cadical] is the top-1 winner in
the SAT track of the SAT Race 2019.

[satlib]: https://www.cs.ubc.ca/~hoos/SATLIB/
[bench]: https://www.cs.ubc.ca/~hoos/SATLIB/benchm.html
[cadical]: http://fmv.jku.at/cadical/

## References

This lecture note is useful to understand the DPLL algorithm:

- http://www.cs.cornell.edu/courses/cs4860/2009sp/lec-04.pdf

This book has a ton of examples:

- https://sat-smt.codes/
