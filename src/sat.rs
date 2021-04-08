pub type Assignment = Vec<bool>;
pub type Formula = Vec<Clause>;
pub type Clause = Vec<Literal>;

#[derive(Clone, Copy, PartialEq)]
pub enum Literal {
    Var(usize),
    Not(usize),
}

pub use Literal::Not;
pub use Literal::Var;

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

fn unit_propagate(mut formula: &mut Formula, vars: &mut Assignment) {
    while let Some(lit) = find_unit_clause(&formula) {
        let (var, truth) = match lit {
            Var(i) => (i, true),
            Not(i) => (i, false),
        };
        vars[var] = truth;
        assign(&mut formula, var, truth);
    }
}

fn find_unit_clause(formula: &Formula) -> Option<Literal> {
    for clause in formula {
        if clause.len() == 1 {
            return Some(clause[0]);
        }
    }
    None
}

/// Assigns a value to the specified variable and simplifies the formula.
fn assign(formula: &mut Formula, var: usize, truth: bool) {
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

/// Finds the most used variable in a formula. This function allocates O(n_vars)
/// temporary memory.
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
    fn test_dpll() {
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
    fn test_assign() {
        // Raw and negated literals are resolved differently.
        {
            let mut formula = vec![vec![Var(1), Var(2)], vec![Not(1), Var(3)]];
            assign(&mut formula, 1, true);
            assert!(formula == vec![vec![Var(3)]]);
        }

        // Falsey unit clause becomes an empty clause.
        {
            let mut formula = vec![vec![Var(1)], vec![Var(2)]];
            assign(&mut formula, 1, false);
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
