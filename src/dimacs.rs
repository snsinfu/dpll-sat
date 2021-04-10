use super::sat;
use std::fmt;
use std::io;

pub enum Error {
    NoHeader,
    BadHeader,
    BadClause,
    VariableCount,
    ClauseCount,
    IO(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoHeader => write!(f, "no header"),
            Error::BadHeader => write!(f, "bad header"),
            Error::BadClause => write!(f, "bad clause"),
            Error::VariableCount => write!(f, "unexpected number of variables"),
            Error::ClauseCount => write!(f, "unexpected number of clauses"),
            Error::IO(err) => err.fmt(f),
        }
    }
}

/// Loads DIMACS CNF formula.
pub fn load(mut src: &mut dyn io::BufRead) -> Result<sat::Formula, Error> {
    let header = parse_header(&mut src)?;
    let formula = parse_formula(&mut src, &header)?;
    Ok(formula)
}

#[derive(Debug, PartialEq)]
struct Header {
    num_variables: usize,
    num_clauses: usize,
}

fn parse_header(src: &mut dyn io::BufRead) -> Result<Header, Error> {
    let mut line = String::new();

    loop {
        line.clear();
        match src.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(err) => return Err(Error::IO(err)),
        }

        if line.starts_with("c") {
            continue;
        }

        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() == 0 {
            continue;
        }

        // p cnf <num> <num>
        if tokens[0] == "p" {
            if tokens.len() != 4 {
                return Err(Error::BadHeader);
            }

            if tokens[1] != "cnf" {
                return Err(Error::BadHeader);
            }

            let mut header = Header {
                num_variables: 0,
                num_clauses: 0,
            };

            if let Ok(num) = tokens[2].parse::<usize>() {
                header.num_variables = num
            } else {
                return Err(Error::BadHeader);
            }

            if let Ok(num) = tokens[3].parse::<usize>() {
                header.num_clauses = num;
            } else {
                return Err(Error::BadHeader);
            }

            return Ok(header);
        }

        break;
    }

    Err(Error::NoHeader)
}

fn parse_formula(src: &mut dyn io::BufRead, header: &Header) -> Result<sat::Formula, Error> {
    // First, load all numeral tokens from the source.
    let mut line = String::new();
    let mut spec = Vec::<i32>::new();

    loop {
        line.clear();
        match src.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(err) => return Err(Error::IO(err)),
        }

        if line.starts_with("c") {
            continue;
        }

        for token in line.split_whitespace() {
            if let Ok(value) = token.parse::<i32>() {
                spec.push(value);
            } else {
                return Err(Error::BadClause);
            }
        }
    }

    // Then, parse the sequence of numeral tokens as CNF clauses separated by
    // a token '0'.
    let mut formula = sat::Formula::new();
    let mut clause = sat::Clause::new();

    for value in spec {
        if value == 0 {
            formula.push(clause.to_vec());
            clause.clear();
            continue;
        }

        if value.abs() as usize > header.num_variables {
            return Err(Error::VariableCount);
        }

        // Convert one-based signed index to zero-based tagged index we use.
        if value > 0 {
            clause.push(sat::Var((value - 1) as usize));
        } else {
            clause.push(sat::Not((-value - 1) as usize));
        }
    }

    if formula.len() != header.num_clauses {
        return Err(Error::ClauseCount);
    }

    Ok(formula)
}

#[cfg(test)]
mod test {
    use super::*;
    use sat::Not;
    use sat::Var;

    #[test]
    fn test_load() {
        let mut src = "c example\np cnf 3 2\n1 -2 3 0\n-1 -3 0\n".as_bytes();
        let result = load(&mut src);
        let expect = vec![vec![Var(0), Not(1), Var(2)], vec![Not(0), Not(2)]];
        match result {
            Ok(actual) => assert_eq!(actual, expect),
            Err(err) => panic!(format!("unexpected: {}", err)),
        }
    }

    #[test]
    fn test_parse_header_no_header() {
        let mut src = "1 2 3 4\n".as_bytes();
        let result = parse_header(&mut src);
        match result {
            Ok(_) => panic!(),
            Err(err) => match err {
                Error::NoHeader => {}
                _ => panic!(format!("unexpected: {}", err)),
            },
        }
    }

    #[test]
    fn test_parse_header_valid_header() {
        let mut src = "p cnf 3 2\n".as_bytes();
        let result = parse_header(&mut src);
        let expect = Header {
            num_variables: 3,
            num_clauses: 2,
        };
        match result {
            Ok(actual) => assert_eq!(actual, expect),
            Err(err) => panic!(format!("error: {}", err)),
        }
    }

    #[test]
    fn test_parse_header_comment() {
        let mut src = "c comment\np cnf 3 2\n".as_bytes();
        let result = parse_header(&mut src);
        let expect = Header {
            num_variables: 3,
            num_clauses: 2,
        };
        match result {
            Ok(actual) => assert_eq!(actual, expect),
            Err(err) => panic!(format!("error: {}", err)),
        }
    }

    #[test]
    fn test_parse_header_empty_lines() {
        let mut src = "\n\np cnf 3 2\n".as_bytes();
        let result = parse_header(&mut src);
        let expect = Header {
            num_variables: 3,
            num_clauses: 2,
        };
        match result {
            Ok(actual) => assert_eq!(actual, expect),
            Err(err) => panic!(format!("error: {}", err)),
        }
    }

    #[test]
    fn test_parse_header_no_required_fields() {
        let mut src = "p cnf\n".as_bytes();
        let result = parse_header(&mut src);
        match result {
            Ok(_) => panic!(),
            Err(err) => match err {
                Error::BadHeader => {}
                _ => panic!(format!("unexpected: {}", err)),
            },
        }
    }

    #[test]
    fn test_parse_header_extra_fields() {
        let mut src = "p cnf 3 2 1\n".as_bytes();
        let result = parse_header(&mut src);
        match result {
            Ok(_) => panic!(),
            Err(err) => match err {
                Error::BadHeader => {}
                _ => panic!(format!("unexpected: {}", err)),
            },
        }
    }

    #[test]
    fn test_parse_header_not_cnf() {
        let mut src = "p dnf 3 2\n".as_bytes();
        let result = parse_header(&mut src);
        match result {
            Ok(_) => panic!(),
            Err(err) => match err {
                Error::BadHeader => {}
                _ => panic!(format!("unexpected: {}", err)),
            },
        }
    }

    #[test]
    fn test_parse_header_negative_variables() {
        let mut src = "p cnf -1 2\n".as_bytes();
        let result = parse_header(&mut src);
        match result {
            Ok(_) => panic!(),
            Err(err) => match err {
                Error::BadHeader => {}
                _ => panic!(format!("unexpected: {}", err)),
            },
        }
    }

    #[test]
    fn test_parse_header_negative_clauses() {
        let mut src = "p cnf 1 -2\n".as_bytes();
        let result = parse_header(&mut src);
        match result {
            Ok(_) => panic!(),
            Err(err) => match err {
                Error::BadHeader => {}
                _ => panic!(format!("unexpected: {}", err)),
            },
        }
    }

    #[test]
    fn test_parse_formula_empty() {
        let mut src = "".as_bytes();
        let header = Header {
            num_variables: 0,
            num_clauses: 0,
        };
        let result = parse_formula(&mut src, &header);
        let expect = sat::Formula::new();
        match result {
            Ok(actual) => assert_eq!(actual, expect),
            Err(err) => panic!(format!("unexpected: {}", err)),
        }
    }

    #[test]
    fn test_parse_formula_valid() {
        let mut src = "1 2 0\nc comment\n-3 -4 -5 0\n".as_bytes();
        let header = Header {
            num_variables: 5,
            num_clauses: 2,
        };
        let result = parse_formula(&mut src, &header);
        let expect = vec![vec![Var(0), Var(1)], vec![Not(2), Not(3), Not(4)]];
        match result {
            Ok(actual) => assert_eq!(actual, expect),
            Err(err) => panic!(format!("unexpected: {}", err)),
        }
    }

    #[test]
    fn test_parse_formula_coalesced() {
        let mut src = "1 2 0 -1 -2 0\n".as_bytes();
        let header = Header {
            num_variables: 2,
            num_clauses: 2,
        };
        let result = parse_formula(&mut src, &header);
        let expect = vec![vec![Var(0), Var(1)], vec![Not(0), Not(1)]];
        match result {
            Ok(actual) => assert_eq!(actual, expect),
            Err(err) => panic!(format!("unexpected: {}", err)),
        }
    }

    #[test]
    fn test_parse_formula_too_many_variables() {
        let mut src = "1 2 3 4 0\n".as_bytes();
        let header = Header {
            num_variables: 3,
            num_clauses: 1,
        };
        let result = parse_formula(&mut src, &header);
        match result {
            Ok(_) => panic!(),
            Err(err) => match err {
                Error::VariableCount => {}
                _ => panic!(format!("unexpected: {}", err)),
            },
        }
    }

    #[test]
    fn test_parse_formula_too_many_clauses() {
        let mut src = "1 2 0 1 2 0 1 2 0\n".as_bytes();
        let header = Header {
            num_variables: 2,
            num_clauses: 2,
        };
        let result = parse_formula(&mut src, &header);
        match result {
            Ok(_) => panic!(),
            Err(err) => match err {
                Error::ClauseCount => {}
                _ => panic!(format!("unexpected: {}", err)),
            },
        }
    }

    #[test]
    fn test_parse_formula_too_less_clauses() {
        let mut src = "1 2 0\n".as_bytes();
        let header = Header {
            num_variables: 2,
            num_clauses: 2,
        };
        let result = parse_formula(&mut src, &header);
        match result {
            Ok(_) => panic!(),
            Err(err) => match err {
                Error::ClauseCount => {}
                _ => panic!(format!("unexpected: {}", err)),
            },
        }
    }
}
