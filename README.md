# dpll-sat

[![Build Status][build-badge]][build-url]

[build-badge]: https://github.com/snsinfu/dpll-sat/workflows/test/badge.svg
[build-url]: https://github.com/snsinfu/dpll-sat/actions?query=workflow%3Atest

**dpll-sat** is a SAT solver implementing the classic [DPLL algorithm][dpll]. I
wrote this program for learning purposes and also for comparing the performance
of a naive DPLL solver and modern solvers.

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

## References

This lecture note is useful to understand the DPLL algorithm:

- http://www.cs.cornell.edu/courses/cs4860/2009sp/lec-04.pdf

This book has a ton of examples:

- https://sat-smt.codes/

This page has many DIMACS CNF files to play with:

- https://www.cs.ubc.ca/~hoos/SATLIB/benchm.html
