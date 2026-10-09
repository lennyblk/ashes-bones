//!   format d'échange
//!   ACT 3 5 7       l'unité 3 va en (5,7)
//!   ACT 3 5 7 A 9   ... puis attaque l'unité 9 (H 9 = soigne)
//!   WAIT 3
//!   END

use std::io::{self, BufRead, BufReader, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use crate::command::Command;
use crate::unit::PendingAction;

/// ce que la boucle du jeu récupère de la connexion
pub enum Event {
    Command(Command),
    Disconnected,
}

/// connexion ouverte avec l'adversaire. Un thread lit le socket et dépose ce qu'il
/// reçoit dans un channel : la boucle du jeu le récupère sans jamais bloquer (poll)
pub struct Connection {
    stream: TcpStream,
    events: Receiver<Event>,
}

impl Connection {
    /// lance le thread lecteur sur une connexion déjà ouverte
    pub fn start(stream: TcpStream) -> io::Result<Connection> {
        // petits messages : on les envoie tout de suite au lieu d'attendre d'en grouper
        stream.set_nodelay(true)?;
        let reader = BufReader::new(stream.try_clone()?);
        let (sender, events) = mpsc::channel();

        thread::spawn(move || {
            for line in reader.lines() {
                // ligne illisible ou erreur : on coupe plutôt que de laisser les parties diverger
                let Some(command) = line.ok().and_then(|l| decode(&l)) else {
                    break;
                };
                if sender.send(Event::Command(command)).is_err() {
                    return; // la Connection a été jetée, plus personne n'écoute
                }
            }
            // fin de la boucle = l'autre a fermé, ou la connexion a lâché
            let _ = sender.send(Event::Disconnected);
        });

        Ok(Connection { stream, events })
    }

    /// false si l'envoi a échoué (adversaire parti)
    pub fn send(&self, command: Command) -> bool {
        writeln!(&self.stream, "{}", encode(command)).is_ok()
    }

    /// prochain événement reçu, ou None s'il n'y a rien : ne bloque jamais
    pub fn poll(&self) -> Option<Event> {
        match self.events.try_recv() {
            Ok(event) => Some(event),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Event::Disconnected),
        }
    }
}

impl Drop for Connection {
    // le thread lecteur garde sa propre copie du socket : sans shutdown il resterait
    // ouvert, et l'adversaire ne saurait jamais qu'on est parti
    fn drop(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
    }
}

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
