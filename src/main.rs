use std::io;

mod dimacs;
mod sat;

fn main() {
    let formula = match dimacs::load(&mut io::stdin().lock()) {
        Ok(formula) => formula,
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(1);
        }
    };

    if let Some(vars) = sat::check_sat(&formula) {
        println!("{}", format_assignment(&vars));
    } else {
        std::process::exit(1);
    }
}

fn format_assignment(vars: &sat::Assignment) -> String {
    let mut message = String::new();
    for (i, &truth) in vars.iter().enumerate() {
        if i > 0 {
            message.push(' ');
        }
        message.push_str(&format!("{}{}", if truth { "" } else { "-" }, i + 1));
    }
    message
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_format_assignment_empty() {
        let vars = sat::Assignment::new();
        let actual = format_assignment(&vars);
        let expect = "";
        assert_eq!(actual, expect);
    }

    #[test]
    fn test_format_assignment_one_based_signed() {
        let vars = vec![true, false, true, false];
        let actual = format_assignment(&vars);
        let expect = "1 -2 3 -4";
        assert_eq!(actual, expect);
    }
}
