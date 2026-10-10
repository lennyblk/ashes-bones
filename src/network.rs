//!   format d'échange
//!   START human 3 4 1 7 ...   faction de l'hôte, puis la case de chaque unité
//!   ACT 3 5 7       l'unité 3 va en (5,7)
//!   ACT 3 5 7 A 9   ... puis attaque l'unité 9 (H 9 = soigne)
//!   WAIT 3
//!   END
//!   DUEL PERFECT    résultat de mon mini-jeu pour le duel en cours (GOOD / BAD)
//!   RETRY           revanche demandée en fin de partie
//!
//!   découverte en LAN (UDP broadcast, port 6667), annoncée par l'hôte toutes les secondes :
//!   ASHES|human|open|nom de la partie|nom de l'hôte      (open / full)

use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::net::{IpAddr, Shutdown, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use crate::command::Command;
use crate::minigame::TimingResult;
use crate::unit::{Faction, PendingAction};

pub const PORT: u16 = 6666;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// tout ce qui passe dans le tuyau
pub enum Message {
    /// début de partie (hôte -> invité) : faction choisie par l'hôte et case de chaque
    /// unité, dans l'ordre de game.units
    Start {
        host_faction: Faction,
        positions: Vec<(i32, i32)>,
    },
    Command(Command),
    Duel(TimingResult),
    /// revanche demandée en fin de partie
    Retry,
}

/// ce que la boucle du jeu récupère de la connexion
pub enum Event {
    Message(Message),
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
                let Some(message) = line.ok().and_then(|l| decode(&l)) else {
                    break;
                };
                if sender.send(Event::Message(message)).is_err() {
                    return; // la Connection a été jetée, plus personne n'écoute
                }
            }
            // fin de la boucle = l'autre a fermé, ou la connexion a lâché
            let _ = sender.send(Event::Disconnected);
        });

        Ok(Connection { stream, events })
    }

    /// false si l'envoi a échoué (adversaire parti)
    pub fn send(&self, message: &Message) -> bool {
        writeln!(&self.stream, "{}", encode(message)).is_ok()
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

/// l'hôte attend son adversaire sans geler la fenêtre. Le jeter annule l'attente
pub struct Host {
    listener: TcpListener,
}

impl Host {
    pub fn new() -> io::Result<Host> {
        // 0.0.0.0 = accepter les connexions qui arrivent par n'importe quelle carte réseau
        let listener = TcpListener::bind(("0.0.0.0", PORT))?;
        // accept() répondra « personne » tout de suite au lieu d'attendre
        listener.set_nonblocking(true)?;
        Ok(Host { listener })
    }

    /// à appeler à chaque frame : None tant que personne n'est arrivé
    pub fn try_accept(&self) -> Option<io::Result<Connection>> {
        match self.listener.accept() {
            // le tunnel, lui, doit rester bloquant : c'est le thread lecteur qui attend dessus
            Ok((stream, _)) => Some(
                stream
                    .set_nonblocking(false)
                    .and_then(|_| Connection::start(stream)),
            ),
            Err(e) if e.kind() == ErrorKind::WouldBlock => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// l'invité se connecte dans un thread : connect() peut attendre plusieurs secondes
pub struct Join {
    result: Receiver<io::Result<Connection>>,
}

impl Join {
    pub fn new(address: String) -> Join {
        let (sender, result) = mpsc::channel();
        thread::spawn(move || {
            let _ = sender.send(connect(&address));
        });
        Join { result }
    }

    /// à appeler à chaque frame : None tant que la tentative est en cours
    pub fn try_connect(&self) -> Option<io::Result<Connection>> {
        match self.result.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                Some(Err(io::Error::other("tentative de connexion interrompue")))
            }
        }
    }
}

fn connect(address: &str) -> io::Result<Connection> {
    // "192.168.1.20" + le port -> adresse réseau utilisable par connect
    let target = (address, PORT)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "adresse introuvable"))?;
    let stream = TcpStream::connect_timeout(&target, CONNECT_TIMEOUT)?;
    Connection::start(stream)
}

/// Message -> ligne de texte à envoyer
fn encode(message: &Message) -> String {
    match message {
        Message::Start {
            host_faction,
            positions,
        } => {
            let coords: Vec<String> = positions.iter().map(|(x, y)| format!("{x} {y}")).collect();
            format!("START {} {}", faction_name(*host_faction), coords.join(" "))
        }
        Message::Command(command) => encode_command(*command),
        Message::Retry => String::from("RETRY"),
        Message::Duel(result) => format!(
            "DUEL {}",
            match result {
                TimingResult::Bad => "BAD",
                TimingResult::Good => "GOOD",
                TimingResult::Perfect => "PERFECT",
            }
        ),
    }
}

/// ligne reçue -> Message. None si la ligne ne respecte pas le format
fn decode(line: &str) -> Option<Message> {
    let words: Vec<&str> = line.split_whitespace().collect();
    if let ["START", faction, coords @ ..] = words.as_slice() {
        let numbers: Vec<i32> = coords
            .iter()
            .map(|w| w.parse().ok())
            .collect::<Option<_>>()?;
        if !numbers.len().is_multiple_of(2) {
            return None;
        }
        return Some(Message::Start {
            host_faction: parse_faction(faction)?,
            positions: numbers.chunks(2).map(|p| (p[0], p[1])).collect(),
        });
    }
    if let ["RETRY"] = words.as_slice() {
        return Some(Message::Retry);
    }
    if let ["DUEL", result] = words.as_slice() {
        return Some(Message::Duel(match *result {
            "BAD" => TimingResult::Bad,
            "GOOD" => TimingResult::Good,
            "PERFECT" => TimingResult::Perfect,
            _ => return None,
        }));
    }
    decode_command(&words).map(Message::Command)
}

pub fn faction_name(faction: Faction) -> &'static str {
    match faction {
        Faction::Human => "human",
        Faction::Undead => "undead",
    }
}

pub fn parse_faction(name: &str) -> Option<Faction> {
    match name {
        "human" => Some(Faction::Human),
        "undead" => Some(Faction::Undead),
        _ => None,
    }
}

fn encode_command(command: Command) -> String {
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

fn decode_command(words: &[&str]) -> Option<Command> {
    let command = match words {
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

// découverte des parties en LAN -----------------------------------------------

pub const DISCOVERY_PORT: u16 = 6667;
const ANNOUNCE_EVERY: Duration = Duration::from_secs(1);
const FORGET_AFTER: Duration = Duration::from_secs(3);

/// ce qu'un hôte annonce sur le réseau local
#[derive(Clone)]
pub struct GameInfo {
    pub name: String,
    pub host_name: String,
    pub host_faction: Faction,
    pub full: bool,
}

/// hôte : annonce sa partie toutes les secondes à tout le réseau local (broadcast UDP).
/// Continue pendant la partie avec full = true, pour qu'elle s'affiche en FULL
pub struct Announcer {
    socket: UdpSocket,
    pub info: GameInfo,
    last_sent: Option<Instant>,
}

impl Announcer {
    pub fn new(info: GameInfo) -> io::Result<Announcer> {
        // port 0 : l'OS choisit un port libre pour envoyer, seul le port de destination compte
        let socket = UdpSocket::bind(("0.0.0.0", 0))?;
        socket.set_broadcast(true)?;
        Ok(Announcer {
            socket,
            info,
            last_sent: None,
        })
    }

    /// à appeler à chaque frame : envoie l'annonce si la dernière date d'au moins 1 s
    pub fn tick(&mut self) {
        if self.last_sent.is_some_and(|t| t.elapsed() < ANNOUNCE_EVERY) {
            return;
        }
        let text = encode_info(&self.info);
        let _ = self
            .socket
            .send_to(text.as_bytes(), ("255.255.255.255", DISCOVERY_PORT));
        self.last_sent = Some(Instant::now());
    }
}

/// une partie vue sur le réseau local
pub struct LanGame {
    pub address: IpAddr,
    pub info: GameInfo,
    last_seen: Instant,
}

/// écran Multiplayer : écoute les annonces et tient la liste des parties du réseau local
pub struct Browser {
    socket: UdpSocket,
    games: Vec<LanGame>,
}

impl Browser {
    pub fn new() -> io::Result<Browser> {
        let socket = UdpSocket::bind(("0.0.0.0", DISCOVERY_PORT))?;
        socket.set_nonblocking(true)?;
        Ok(Browser {
            socket,
            games: Vec::new(),
        })
    }

    /// à appeler à chaque frame : lit les annonces arrivées, oublie les hôtes muets
    pub fn update(&mut self) {
        let mut buffer = [0u8; 512];
        while let Ok((size, from)) = self.socket.recv_from(&mut buffer) {
            let Some(info) = std::str::from_utf8(&buffer[..size])
                .ok()
                .and_then(decode_info)
            else {
                continue; // pas une annonce du jeu
            };
            match self.games.iter_mut().find(|g| g.address == from.ip()) {
                Some(game) => {
                    game.info = info;
                    game.last_seen = Instant::now();
                }
                None => self.games.push(LanGame {
                    address: from.ip(),
                    info,
                    last_seen: Instant::now(),
                }),
            }
        }
        self.games.retain(|g| g.last_seen.elapsed() < FORGET_AFTER);
    }

    pub fn games(&self) -> &[LanGame] {
        &self.games
    }

    /// bouton Refresh : on vide, les hôtes encore là réapparaissent dans la seconde
    pub fn clear(&mut self) {
        self.games.clear();
    }
}

fn encode_info(info: &GameInfo) -> String {
    let state = if info.full { "full" } else { "open" };
    format!(
        "ASHES|{}|{state}|{}|{}",
        faction_name(info.host_faction),
        info.name,
        info.host_name
    )
}

fn decode_info(text: &str) -> Option<GameInfo> {
    let parts: Vec<&str> = text.split('|').collect();
    let ["ASHES", faction, state, name, host_name] = parts.as_slice() else {
        return None;
    };
    Some(GameInfo {
        name: name.to_string(),
        host_name: host_name.to_string(),
        host_faction: parse_faction(faction)?,
        full: match *state {
            "open" => false,
            "full" => true,
            _ => return None,
        },
    })
}
