//!   format d'échange
//!   ACT 3 5 7       l'unité 3 va en (5,7)
//!   ACT 3 5 7 A 9   ... puis attaque l'unité 9 (H 9 = soigne)
//!   WAIT 3
//!   END

use crate::command::Command;
use crate::unit::PendingAction;

/// Command -> ligne de texte à envoyer
pub fn encode(command: Command) -> String {
    match command {
        Command::Act { unit, to, action } => {
            let action = match action {
                None => String::new(),
                Some(PendingAction::Attack(target)) => format!(" A {target}"),
                Some(PendingAction::Heal(target)) => format!(" H {target}"),
            };
            format!("ACT {unit} {} {}{action}", to.0, to.1)
        }
        Command::Wait { unit } => format!("WAIT {unit}"),
        Command::EndTurn => String::from("END"),
    }
}

/// ligne reçue -> Command. None si la ligne ne respecte pas le format
pub fn decode(line: &str) -> Option<Command> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let command = match words.as_slice() {
        ["ACT", unit, x, y] => Command::Act {
            unit: unit.parse().ok()?,
            to: (x.parse().ok()?, y.parse().ok()?),
            action: None,
        },
        ["ACT", unit, x, y, kind, target] => {
            let target = target.parse().ok()?;
            Command::Act {
                unit: unit.parse().ok()?,
                to: (x.parse().ok()?, y.parse().ok()?),
                action: Some(match *kind {
                    "A" => PendingAction::Attack(target),
                    "H" => PendingAction::Heal(target),
                    _ => return None,
                }),
            }
        }
        ["WAIT", unit] => Command::Wait {
            unit: unit.parse().ok()?,
        },
        ["END"] => Command::EndTurn,
        _ => return None,
    };
    Some(command)
}
