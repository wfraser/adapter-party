use std::{
    borrow::Cow,
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

#[derive(Debug, Clone, Eq)]
struct Adapter(Thread, Thread, Cow<'static, str>);

impl Adapter {
    pub fn new(a: Thread, b: Thread) -> Self {
        Self(a, b, Cow::Borrowed(""))
    }

    pub fn with_name(self, name: &'static str) -> Self {
        Self(self.0, self.1, Cow::Borrowed(name))
    }

    pub fn reverse(self) -> Self {
        let name = if self.2.is_empty() {
            self.2
        } else if let Some(s) = self.2.strip_suffix(" (reversed)") {
            Cow::Owned(s.to_owned())
        } else {
            Cow::Owned(format!("{} (reversed)", self.2))
        };
        Self(self.1, self.0, name)
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
            f.write_str(&self.2)?;
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
        chain: Chain::new(Adapter::new(NIL_THREAD, start).with_name("start")),
    }];

    let mut found = vec![];

    while let Some(state) = states.pop() {
        for a in equipment {
            if state.used.contains(a) {
                continue;
            }
            if let Some(mut chain) = state.chain.add(a.clone()) {
                if chain.0.last().unwrap().1.opposite() == end {
                    chain.0.push(Adapter::new(end, NIL_THREAD).with_name("end"));
                    found.push(chain);
                } else {
                    let mut used = state.used.clone();
                    used.insert(a.clone());
                    states.push(State { used, chain });
                }
            }
        }
    }

    found
}

/// For all possible adapters (using threads present on existing equipment), how many new chains do
/// they make possible if they are added?
fn find_useful_additions(equipment: &[Adapter]) -> Vec<(Adapter, usize)> {
    let all_threads = equipment.iter()
        .map(|adapter| adapter.0)
        .chain(equipment.iter().map(|adapter| adapter.1))
        .map(|thread| thread.opposite())
        .collect::<HashSet<Thread>>();
    
    let all_adapters = all_threads.iter()
        .flat_map(|a| {
            all_threads.iter()
                .map(|b| Adapter::new(*a, *b))
        })
        .collect::<HashSet<Adapter>>();

    fn count_chains(pairs: impl Iterator<Item=(Thread, Thread)>, equipment: &[Adapter]) -> usize {
        pairs.map(|(a, b)| make_chain(a, b, equipment).len())
            .map(|count| if count == 0 { 0 } else { 1 })
            .sum()
    }

    let start = count_chains(all_adapters.iter().map(|a| (a.0, a.1)), equipment);

    let mut results = vec![];
    let mut new_equip = equipment.to_vec();
    for new in &all_adapters {
        new_equip.push(new.clone());
        let count = count_chains(all_adapters.iter().map(|a| (a.0, a.1)), &new_equip);
        results.push((new.clone(), count - start));
        new_equip.pop();
    }

    results.sort_by_key(|(_a, n)| *n);
    results
}

fn main() {
    use Thread::*;

    // All the random crap I own:
    let mut equipment = vec![
        // Mount adapters:
        Adapter::new(M("EF"), F("58")),
        Adapter::new(M("EF"), F("LTM")),
        Adapter::new(M("EF"), F("M42")),
        Adapter::new(M("EF"), F("FD")),

        // Gender / thread changers:
        Adapter::new(M("58"), M("58")),
        Adapter::new(M("Bay1"), F("46mm")),

        // Step-up rings:
        Adapter::new(M("40.5"), F("46")),
        Adapter::new(M("46"), F("52")),
        Adapter::new(M("46"), F("77")),
        Adapter::new(M("52"), F("77")),
        Adapter::new(M("55"), F("77")),
        Adapter::new(M("58"), F("77")),
        Adapter::new(M("62"), F("77")),
        Adapter::new(M("72"), F("77")),

        // Step-down rings:
        Adapter::new(M("72"), F("52")),
        Adapter::new(M("58"), F("52")),

        // Lenses:
        Adapter::new(M("LTM"), F("40.5")).with_name("Rodenstock Rodagon 50mm f/2.8"),
        Adapter::new(M("LTM"), F("42")).with_name("Schneider Componon-S 80mm f/4"),
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

    println!("---");
    // If I add this new piece, can I get one of the enlarger lenses on backwards?
    equipment.push(Adapter::new(M("52"), F("58")).with_name("new 52-58"));
    let chains = make_chain(
        F("EF"),
        F("LTM"),
        &equipment,
    );
    for chain in chains {
        println!("{}", chain);
    }

    println!("---");
    for (adapter, count) in find_useful_additions(&equipment) {
        println!("{}: {} new chains", adapter, count);
    }
}
