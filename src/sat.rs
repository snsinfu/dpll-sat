/// Variable assignment for a SAT problem. The i-th element designates the truth
/// value of the i-th variable.
pub type Assignment = Vec<bool>;

/// CNF formula. A formula represents the AND-product of the contained clauses.
pub type Formula = Vec<Clause>;

/// A clause in a CNF formula. A clause represents the OR-sum of the contained
/// literals.
pub type Clause = Vec<Literal>;

/// A literal in a clause.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Literal {
    Var(usize),
    Not(usize),
}

pub use Literal::Not;
pub use Literal::Var;

/// Solves a satisfiability problem given as a CNF formula.
///
/// Returns a variable assignment if the formula is satisfiable, or None if the
/// formula is unsatisfiable.
pub fn check_sat(formula: &Formula) -> Option<Assignment> {
    let mut n_vars = 0;

    for clause in formula {
        for lit in clause {
            let i = match lit {
                &Var(i) => i,
                &Not(i) => i,
            };
            if i >= n_vars {
                n_vars = i + 1;
            }
        }
    }

    let mut vars = vec![false; n_vars];

    if dpll(&formula, &mut vars) {
        Some(vars)
    } else {
        None
    }
}

fn dpll(formula: &Formula, mut vars: &mut Assignment) -> bool {
    let mut formula = formula.to_vec();

    unit_propagate(&mut formula, &mut vars);

    if formula.is_empty() {
        return true;
    }

    if formula.iter().any(|clause| clause.is_empty()) {
        return false;
    }

    // Simple splitting rule: Assign a truth value to the most used variable
    // in the formula.
    let var = find_dominant_variable(&formula, vars.len());

    formula.push(vec![Var(var)]);
    if dpll(&formula, &mut vars) {
        return true;
    }

    formula.pop();
    formula.push(vec![Not(var)]);
    dpll(&formula, &mut vars)
}

/// Resolves unit clauses in a CNF formula.
///
/// # Unit propagation
///
/// A unit clause is a clause consisting of a single literal:
///
/// > formula = ... ∧ x ∧ ...
///
/// Such a clause in a CNF formula induces an assignment `x = true` so that the
/// formula must become true. Unit propagation finds out such assignments and
/// simplifies the formula until all unit clauses are consumed.
///
fn unit_propagate(mut formula: &mut Formula, vars: &mut Assignment) {
    while let Some(clause) = formula.iter().find(|clause| clause.len() == 1) {
        let (var, truth) = match clause[0] {
            Var(i) => (i, true),
            Not(i) => (i, false),
        };
        vars[var] = truth;
        simplify(&mut formula, var, truth);
    }
}

/// Simplifies a CNF formula using given variable assignment.
///
/// # Simplification
///
/// A variable assignment resolves literals `Var(var)` and `Not(var)` in the
/// formula to true or false. Note that a false literal (say x) does not
/// contribute to the condition of a clause:
///
/// > C ∨ x = C  if x = false
///
/// so, a simplification will remove all false literals from the formula. On the
/// other hand, a true literal (say y) makes the clause containing y true:
///
/// > C ∨ y = true  if y = true
///
/// i.e., the clause becomes true. So, a simplification will remove all clauses
/// containing one or more true literals.
///
/// # Empty clause
///
/// The simplification may leave empty clause(s) in the formula. An empty clause
/// must have originated from a unit clause consisting of a false literal x:
///
/// > formula = ... ∧ x ∧ ... ,  x = false .
///
/// Therefore, the formula must be unsatisfiable in that case.
///
fn simplify(formula: &mut Formula, var: usize, truth: bool) {
    let truthy_lit = if truth { Var(var) } else { Not(var) };
    let falsey_lit = if truth { Not(var) } else { Var(var) };

    // This function is the hottest part of the solver. So, we loop over
    // clauses and literals manually and remove determined ones in-place.
    let mut clause_index: usize = 0;
    while clause_index < formula.len() {
        let clause = &mut formula[clause_index];

        // Remove unconditionally true clause.
        if clause.contains(&truthy_lit) {
            let n = formula.len();
            formula.swap(clause_index, n - 1);
            formula.pop();
            continue;
        }

        // Remove falsified literal.
        let mut literal_index = 0;
        while literal_index < clause.len() {
            if clause[literal_index] == falsey_lit {
                let n = clause.len();
                clause.swap(literal_index, n - 1);
                clause.pop();
                continue;
            }
            literal_index += 1;
        }

        clause_index += 1;
    }
}

/// Finds the most used variable in a formula.
fn find_dominant_variable(formula: &Formula, n_vars: usize) -> usize {
    let mut freqs = vec![0; n_vars];

    for clause in formula {
        for lit in clause {
            let i = match lit {
                &Var(i) => i,
                &Not(i) => i,
            };
            freqs[i] += 1;
        }
    }

    let mut max: i32 = 0;
    let mut argmax: usize = 0;

    for (i, &freq) in freqs.iter().enumerate() {
        if freq > max {
            max = freq;
            argmax = i;
        }
    }

    return argmax;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_check_sat() {
        // Empty formula
        {
            let formula = vec![];
            let sat = check_sat(&formula);

            assert!(sat == Some(vec![]));
        }

        // Satisfiable example
        {
            let formula = vec![
                vec![Var(0), Var(0), Var(1)],
                vec![Not(0), Not(1), Not(1)],
                vec![Not(0), Var(1), Var(1)],
            ];
            let sat = check_sat(&formula);

            assert!(sat == Some(vec![false, true]));
        }

        // Unsatisfiable example
        {
            let formula = vec![
                vec![Var(0), Var(1)],
                vec![Not(0), Not(1)],
                vec![Var(1), Var(2)],
                vec![Not(1), Not(2)],
                vec![Var(2), Var(0)],
                vec![Not(2), Not(0)],
            ];
            let sat = check_sat(&formula);

            assert!(sat == None);
        }
    }

    #[test]
    fn test_unit_propagate() {
        let mut formula = vec![
            vec![Var(1)],                 // (unit clause)
            vec![Not(2)],                 // (unit clause)
            vec![Var(1), Var(2)],         // => true
            vec![Not(1), Var(2), Var(3)], // => 3 (unit clause)
            vec![Var(0), Not(3), Var(4)], // => 0 | 4
        ];
        let mut vars = vec![false; 5];

        unit_propagate(&mut formula, &mut vars);

        assert!(formula == vec![vec![Var(0), Var(4)]]);
        assert!(vars == vec![false, true, false, true, false]);
    }

    #[test]
    fn test_simplify() {
        // Raw and negated literals are resolved differently.
        {
            let mut formula = vec![vec![Var(1), Var(2)], vec![Not(1), Var(3)]];
            simplify(&mut formula, 1, true);
            assert!(formula == vec![vec![Var(3)]]);
        }

        // Falsey unit clause becomes an empty clause.
        {
            let mut formula = vec![vec![Var(1)], vec![Var(2)]];
            simplify(&mut formula, 1, false);
            assert!(formula == vec![vec![], vec![Var(2)]]);
        }
    }

    #[test]
    fn test_find_dominant_variable() {
        let formula = vec![
            vec![Var(0), Var(1), Var(2)],
            vec![Not(0), Var(1)],
            vec![Not(1), Var(2)],
            vec![Var(0), Not(1), Not(2)],
        ];
        assert!(find_dominant_variable(&formula, 3) == 1);
    }
}
