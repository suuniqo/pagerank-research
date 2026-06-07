use std::{
    collections::HashMap, fs::File, io::{BufRead, BufReader}
};

pub mod error;

pub use error::ParseError;

#[derive(Debug, Clone)]
pub struct GraphMTX {
    pub edges: Vec<(usize, usize)>,
    pub nrows: usize,
    pub ncols: usize,
    pub nnz: usize,
}

impl GraphMTX {
    pub fn new(edges: Vec<(usize, usize)>, nrows: usize, ncols: usize, nnz: usize) -> Self {
        Self {
            edges,
            nrows,
            ncols,
            nnz,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphTSV {
    pub ids: HashMap<String, usize>,
    pub nodes: Vec<String>,
    pub edges: Vec<(usize, usize)>,
    pub categories: Vec<Vec<Vec<String>>>,
}

impl GraphTSV {
    pub fn new(
        ids: HashMap<String, usize>,
        nodes: Vec<String>,
        edges: Vec<(usize, usize)>,
        categories: Vec<Vec<Vec<String>>>,
    ) -> Self {
        Self {
            ids,
            nodes,
            edges,
            categories,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Neuron {
    pub uid: usize,
    pub class: NeuronClass,
    pub region: NeuronRegion
}

impl Neuron {
    pub fn new(uid: usize, class: NeuronClass, region: NeuronRegion) -> Self {
        Self {
            uid,
            class,
            region
        }
    }
}

impl std::hash::Hash for Neuron {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}

impl PartialEq for Neuron {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for Neuron {}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeuronClass {
    Excitatory(ExcitatoryType),
    Inhibitory(InhibitoryType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExcitatoryType {
    L2(NeuronCluster),
    L3(NeuronCluster),
    L4(NeuronCluster),
    L5(NeuronCluster),
    L6(NeuronSpan, NeuronCluster),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeuronCluster {
    A,
    B,
    C,
    ET,
    NP,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeuronSpan {
    Short,
    Tall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InhibitoryType {
    DTC,
    ITC,
    PTC,
    STC,
}

impl NeuronClass {
    fn build(classification: &str, subtype: &str) -> Option<NeuronClass> {
        Some(match classification {
            "excitatory_neuron" => {
                let subtype = ExcitatoryType::from(subtype)?;
                Self::Excitatory(subtype)
            },
            "inhibitory_neuron" => {
                let subtype = InhibitoryType::from(subtype)?;
                Self::Inhibitory(subtype)
            },
            _ => return None,
        })
    }

    pub fn classification(&self) -> String {
        match self {
            NeuronClass::Excitatory(_) => "excitatory_neuron",
            NeuronClass::Inhibitory(_) => "inhibitory_neuron",
        }.to_string()
    }

    pub fn layer(&self) -> String {
        match self {
            NeuronClass::Excitatory(subtype) => subtype.partial_id(),
            NeuronClass::Inhibitory(subtype) => subtype.id(),
        }
    }

    pub fn subtype(&self) -> String {
        match self {
            NeuronClass::Excitatory(subtype) => subtype.id(),
            NeuronClass::Inhibitory(subtype) => subtype.id(),
        }
    }
}

impl InhibitoryType {
    fn from(subtype: &str) -> Option<InhibitoryType> {
        Some(match subtype {
            "DTC" => Self::DTC,
            "ITC" => Self::ITC,
            "PTC" => Self::PTC,
            "STC" => Self::STC,
            _     => return None,
        })
    }

    fn id(&self) -> String {
        match self {
            InhibitoryType::DTC => "DTC",
            InhibitoryType::ITC => "ITC",
            InhibitoryType::PTC => "PTC",
            InhibitoryType::STC => "STC",
        }.to_string()
    }
}

impl ExcitatoryType {
    fn from(subtype: &str) -> Option<ExcitatoryType> {
        if subtype.len() <= 2 {
            return None;
        }

        let layer = &subtype[..2];
        let cluster = &subtype[2..];

        Some(match layer {
            "L2" => {
                let cluster = NeuronCluster::from(cluster)?;
                Self::L2(cluster)
            },
            "L3" => {
                let cluster = NeuronCluster::from(cluster)?;
                Self::L3(cluster)

            },
            "L4" => {
                let cluster = NeuronCluster::from(cluster)?;
                Self::L4(cluster)

            },
            "L5" => {
                let cluster = NeuronCluster::from(cluster)?;
                Self::L5(cluster)
            },
            "L6" => {
                let (span, cluster) = cluster.split_once("-")?;

                let span = NeuronSpan::from(span)?;
                let cluster = NeuronCluster::from(cluster)?;

                Self::L6(span, cluster)
            },
            _ => return None,
        })
    }

    fn partial_id(&self) -> String {
        match self {
            Self::L2(_)  => "L2",
            Self::L3(_)  => "L3",
            Self::L4(_)  => "L4",
            Self::L5(_)  => "L5",
            Self::L6(..) => "L6",
        }.to_string()
    }

    fn id(&self) -> String {
        match self {
            Self::L2(cluster) => format!("{}{}", self.partial_id(), cluster.id()),
            Self::L3(cluster) => format!("{}{}", self.partial_id(), cluster.id()),
            Self::L4(cluster) => format!("{}{}", self.partial_id(), cluster.id()),
            Self::L5(cluster) => format!("{}{}", self.partial_id(), cluster.id()),
            Self::L6(span, cluster) => format!("{}{}-{}", self.partial_id(), span.id(), cluster.id()),
        }
    }
}

impl NeuronCluster {
    fn from(cluster: &str) -> Option<Self> {
        Some(match cluster {
            "a"  => Self::A,
            "b"  => Self::B,
            "c"  => Self::C,
            "ET" => Self::ET,
            "NP" => Self::NP,
            _    => return None,
        })
    }

    fn id(&self) -> String {
        match self {
            Self::A  => "a",
            Self::B  => "b",
            Self::C  => "c",
            Self::ET => "ET",
            Self::NP => "NP",
        }.to_string()
    }
}

impl NeuronSpan {
    fn from(cluster: &str) -> Option<Self> {
        Some(match cluster {
            "short" => Self::Short,
            "tall"  => Self::Tall,
            _       => return None,
        })
    }

    fn id(&self) -> String {
        match self {
            NeuronSpan::Short => "short",
            NeuronSpan::Tall  => "tall",
        }.to_string()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NeuronRegion {
    VISp,
    VISrl,
    VISlm,
    VISal,
}

impl TryFrom<&str> for NeuronRegion {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            "visp" => Ok(Self::VISp),
            "visrl" => Ok(Self::VISrl),
            "vislm" => Ok(Self::VISlm),
            "visal" => Ok(Self::VISal),
            _ => Err(())
        }
    }
}

pub struct GraphMICrONS {
    pub edges: Vec<(usize, usize, usize)>,
    pub neurons: Vec<Neuron>,
    pub ids: HashMap<Neuron, usize>,
}

impl GraphMICrONS {
    pub fn new(edges: Vec<(usize, usize, usize)>, neurons: Vec<Neuron>, ids: HashMap<Neuron, usize>) -> Self {
        Self {
            edges,
            neurons,
            ids
        }
    }
}

pub struct Parser;

impl Parser {
    /// Skips header and keeps in buf the first line that doesnt start with `sym`
    pub fn skip_header(
        reader: &mut BufReader<File>,
        buf: &mut String,
        sym: char,
    ) -> Result<usize, ParseError> {
        loop {
            buf.clear();

            let nbytes = reader.read_line(buf).map_err(ParseError::Io)?;

            if nbytes == 0 {
                return Err(ParseError::EmptyBody);
            }

            let line = buf.trim();

            if line.is_empty() {
                continue;
            }

            if !line.starts_with(sym) {
                return Ok(nbytes);
            }
        }
    }

    pub fn parse_tsv(
        path_articles: &str,
        path_categories: &str,
        path_links: &str,
    ) -> Result<GraphTSV, ParseError> {
        let mut buf = String::new();

        // parse articles
        let mut ids = HashMap::new();
        let mut nodes = Vec::new();

        let file_articles = File::open(path_articles).map_err(ParseError::Io)?;

        let mut reader = BufReader::new(file_articles);

        let _ = Parser::skip_header(&mut reader, &mut buf, '#')?;

        loop {
            let line = buf.trim();

            if line.is_empty() {
                return Err(ParseError::BadLine(buf));
            }

            ids.insert(line.to_string(), nodes.len());
            nodes.push(line.to_string());

            buf.clear();

            let nbytes = reader.read_line(&mut buf).map_err(ParseError::Io)?;

            if nbytes == 0 {
                break;
            }
        }

        // parse categories
        let mut categories = vec![vec![]; nodes.len()];

        let file_categories = File::open(path_categories).map_err(ParseError::Io)?;

        let mut reader = BufReader::new(file_categories);

        let _ = Parser::skip_header(&mut reader, &mut buf, '#')?;

        loop {
            let line = buf.trim();

            if line.is_empty() {
                return Err(ParseError::BadLine(buf));
            }

            let (name, category) = line
                .split_once('\t')
                .ok_or(ParseError::BadLine(buf.clone()))?;

            let name_id = ids.get(name).ok_or(ParseError::Inconsistent {
                reason: format!("name {name} not found in nodes"),
                line: buf.clone(),
            })?;

            categories[*name_id].push(category.split('.').skip(1).map(|s| s.to_string()).collect());

            buf.clear();

            let nbytes = reader.read_line(&mut buf).map_err(ParseError::Io)?;

            if nbytes == 0 {
                break;
            }
        }

        // parse edges
        let file_links = File::open(path_links).map_err(ParseError::Io)?;

        let mut reader = BufReader::new(file_links);

        let mut edges = Vec::new();

        let _ = Parser::skip_header(&mut reader, &mut buf, '#')?;

        loop {
            let line = buf.trim();

            if line.is_empty() {
                return Err(ParseError::BadLine(buf));
            }

            let (src, dst) = line
                .split_once('\t')
                .ok_or(ParseError::BadLine(buf.clone()))?;

            let src_id = ids.get(src).ok_or(ParseError::Inconsistent {
                reason: format!("name {src} not found in nodes"),
                line: buf.clone(),
            })?;

            let dst_id = ids.get(dst).ok_or(ParseError::Inconsistent {
                reason: format!("name {dst} not found in nodes"),
                line: buf.clone(),
            })?;

            edges.push((*src_id, *dst_id));

            buf.clear();

            let nbytes = reader.read_line(&mut buf).map_err(ParseError::Io)?;

            if nbytes == 0 {
                break;
            }
        }

        Ok(GraphTSV::new(ids, nodes, edges, categories))
    }

    pub fn parse_mtx(path: &str) -> Result<GraphMTX, ParseError> {
        let file = File::open(path).map_err(ParseError::Io)?;

        let mut reader = BufReader::new(file);
        let mut buf = String::new();

        // skip header
        let _ = Parser::skip_header(&mut reader, &mut buf, '%')?;

        let mut split = buf.split_whitespace();

        let nrows = split
            .next()
            .ok_or(ParseError::BadLine(buf.clone()))?
            .parse()
            .map_err(|_| ParseError::BadLine(buf.clone()))?;

        let ncols = split
            .next()
            .ok_or(ParseError::BadLine(buf.clone()))?
            .parse()
            .map_err(|_| ParseError::BadLine(buf.clone()))?;

        let nnz = split
            .next()
            .ok_or(ParseError::BadLine(buf.clone()))?
            .parse()
            .map_err(|_| ParseError::BadLine(buf.clone()))?;

        let mut edges = Vec::with_capacity(nnz);

        // parse edges
        for i in 0..nnz {
            buf.clear();

            let nbytes = reader.read_line(&mut buf).map_err(ParseError::Io)?;

            if nbytes == 0 {
                return Err(ParseError::TooShort {
                    expected: nnz,
                    got: i,
                });
            }

            let (src, dst) = buf
                .trim()
                .split_once(' ')
                .ok_or(ParseError::BadLine(buf.clone()))?;

            edges.push((
                src.parse::<usize>()
                    .map_err(|_| ParseError::BadLine(buf.clone()))?
                    - 1,
                dst.parse::<usize>()
                    .map_err(|_| ParseError::BadLine(buf.clone()))?
                    - 1,
            ));
        }

        Ok(GraphMTX::new(edges, nrows, ncols, nnz))
    }

    pub fn parse_microns(path_links: &str, path_neurons: &str) -> Result<GraphMICrONS, ParseError> {
        let mut buf = String::new();

        // parse neurons
        let mut ids = HashMap::new();
        let mut neurons = Vec::new();

        let file_articles = File::open(path_neurons).map_err(ParseError::Io)?;

        let mut reader = BufReader::new(file_articles);

        let _ = Parser::skip_header(&mut reader, &mut buf, '#')?;

        loop {
            let line = buf.trim();

            if line.is_empty() {
                return Err(ParseError::BadLine(buf));
            }

            let mut split = line.split('\t');

            let index = split.next()
                .ok_or(ParseError::BadLine(line.to_string()))?
                .parse::<usize>()
                .map_err(|_| ParseError::BadLine(line.to_string()))?;

            let uid = split.next()
                .ok_or(ParseError::BadLine(line.to_string()))?
                .parse::<usize>()
                .map_err(|_| ParseError::BadLine(line.to_string()))?;

            let cell_subtype = split.next()
                .ok_or(ParseError::BadLine(line.to_string()))?;

            let classification = split.next()
                .ok_or(ParseError::BadLine(line.to_string()))?;

            let region = split.next()
                .ok_or(ParseError::BadLine(line.to_string()))?
                .try_into()
                .map_err(|_| ParseError::BadLine(line.to_string()))?;      

            let cell_type = NeuronClass::build(classification, cell_subtype)
                .ok_or(ParseError::BadLine(line.to_string()))?;
            
            let neuron = Neuron::new(uid, cell_type, region);

            if neurons.len() != index {
                return Err(ParseError::Inconsistent { 
                    reason: "Bad index".to_string(), 
                    line: line.to_string() 
                });
            }

            ids.insert(neuron.clone(), neurons.len());
            neurons.push(neuron);

            buf.clear();

            let nbytes = reader.read_line(&mut buf).map_err(ParseError::Io)?;

            if nbytes == 0 {
                break;
            }
        }

        // parse edges
        let file_links = File::open(path_links).map_err(ParseError::Io)?;

        let mut reader = BufReader::new(file_links);

        let mut edges = Vec::new();

        let _ = Parser::skip_header(&mut reader, &mut buf, '#')?;

        loop {
            let line = buf.trim();

            if line.is_empty() {
                return Err(ParseError::BadLine(buf));
            }

            let mut split = line.split('\t');

            let src = split.next()
                .ok_or(ParseError::BadLine(line.to_string()))?
                .parse::<usize>()
                .map_err(|_| ParseError::BadLine(line.to_string()))?;

            
            let dst = split.next()
                .ok_or(ParseError::BadLine(line.to_string()))?
                .parse::<usize>()
                .map_err(|_| ParseError::BadLine(line.to_string()))?;


            let weight = split.next()
                .ok_or(ParseError::BadLine(line.to_string()))?
                .parse::<usize>()
                .map_err(|_| ParseError::BadLine(line.to_string()))?;

            edges.push((src, dst, weight));

            buf.clear();

            let nbytes = reader.read_line(&mut buf).map_err(ParseError::Io)?;

            if nbytes == 0 {
                break;
            }
        }

        Ok(GraphMICrONS::new(edges, neurons, ids))
    }
}
