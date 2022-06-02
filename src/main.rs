use std::{
    cmp::PartialEq,
    collections::HashSet,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Hash)]
enum Thread {
    M(&'static str),
    F(&'static str),
}

const NIL_THREAD: Thread = Thread::M("nil");

impl Thread {
    pub fn opposite(self) -> Self {
        match self {
            Self::M(x) => Self::F(x),
            Self::F(x) => Self::M(x),
        }
    }
}

impl Display for Thread {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::M(x) => {
                f.write_str(x)?;
                f.write_str("(M)")
            }
            Self::F(x) => {
                f.write_str(x)?;
                f.write_str("(F)")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq)]
struct Adapter(Thread, Thread, &'static str);

impl Adapter {
    pub fn reverse(self) -> Self {
        Self(self.1, self.0, self.2)
    }
}

impl Hash for Adapter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // To allow it to be reversed freely without changing its hash value:
        if self.0 < self.1 {
            self.0.hash(state);
            self.1.hash(state);
        } else {
            self.1.hash(state);
            self.0.hash(state);
        }
    }
}

impl PartialEq for Adapter {
    fn eq(&self, other: &Self) -> bool {
        // Matches the same adapter reversed:
        (self.0 == other.0 && self.1 == other.1)
            || (self.0 == other.1 && self.1 == other.0)
    }
}

impl Display for Adapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !self.2.is_empty() {
            f.write_str(self.2)?;
            if self.0 != NIL_THREAD && self.1 != NIL_THREAD {
                return Ok(());
            } else {
                f.write_str(": ")?;
            }
        }
        if self.0 != NIL_THREAD {
            self.0.fmt(f)?;
            if self.1 != NIL_THREAD {
                f.write_str(" -> ")?;
            }
        }
        if self.1 != NIL_THREAD {
            self.1.fmt(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Chain(Vec<Adapter>);

impl Chain {
    pub fn new(adapter: Adapter) -> Self {
        Self(vec![adapter])
    }

    pub fn add(&self, next: Adapter) -> Option<Self> {
        let last = self.0.last().unwrap().1;
        if last == next.0.opposite() {
            let mut new = self.clone();
            new.0.push(next);
            Some(new)
        } else if last == next.1.opposite() {
            let mut new = self.clone();
            new.0.push(next.reverse());
            Some(new)
        } else {
            None
        }
    }
}

impl Display for Chain {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for a in &self.0 {
            write!(f, "[{}] ", a)?;
        }
        Ok(())
    }
}

fn make_chain(start: Thread, end: Thread, equipment: &[Adapter]) -> Vec<Chain> {
    #[derive(Debug, Clone)]
    struct State {
        used: HashSet<Adapter>,
        chain: Chain,
    }

    let mut states = vec![State {
        used: HashSet::new(),
        chain: Chain::new(Adapter(NIL_THREAD, start, "start")),
    }];

    let mut found = vec![];

    while let Some(state) = states.pop() {
        for a in equipment {
            if state.used.contains(a) {
                continue;
            }
            if let Some(mut chain) = state.chain.add(*a) {
                if chain.0.last().unwrap().1.opposite() == end {
                    chain.0.push(Adapter(end, NIL_THREAD, "end"));
                    found.push(chain);
                } else {
                    let mut used = state.used.clone();
                    used.insert(*a);
                    states.push(State { used, chain });
                }
            }
        }
    }

    found
}

fn main() {
    use Thread::*;

    // All the random crap I own:
    let equipment = vec![
        // Mount adapters:
        Adapter(M("EF"), F("58"), ""),
        Adapter(M("EF"), F("M39"), ""),
        Adapter(M("EF"), F("M42"), ""),
        Adapter(M("EF"), F("FD"), ""),

        // Gender / thread changers:
        Adapter(M("58"), M("58"), ""),
        Adapter(M("Bay1"), F("46mm"), ""),

        // Step-up rings:
        Adapter(M("40.5"), F("46"), ""),
        Adapter(M("46"), F("52"), ""),
        Adapter(M("46"), F("77"), ""),
        Adapter(M("52"), F("77"), ""),
        Adapter(M("55"), F("77"), ""),
        Adapter(M("58"), F("77"), ""),
        Adapter(M("62"), F("77"), ""),
        Adapter(M("72"), F("77"), ""),

        // Step-down rings:
        Adapter(M("72"), F("52"), ""),
        Adapter(M("58"), F("52"), ""),

        // Lenses:
        Adapter(M("M39"), F("40.5"), "Rodenstock Rodagon 50mm f/2.8"),
        Adapter(M("M39"), F("42"), "Schneider Componon-S 80mm f/4"),
    ];

    // EF camera body -> [?? some shit ??] -> 52mm male thread on a slide copier.
    // The correct chain should hopefully involve an enlarger lens.
    let chains = make_chain(
        F("EF"),
        M("52"),
        &equipment,
    );
    for chain in chains {
        println!("{}", chain);
    }
}
