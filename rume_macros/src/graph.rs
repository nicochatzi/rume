use core::ops::Range;
use rume_core::InputEndpointKind;

use proc_macro::{token_stream::IntoIter, TokenStream, TokenTree};

fn consume_until_char(tokens: &mut IntoIter, ch: char) -> String {
    let mut initialiser = String::new();
    loop {
        let token = tokens
            .by_ref()
            .next()
            .expect("Incorrectly terminated declaration");
        if let TokenTree::Punct(token) = token.clone() {
            if token.as_char() == ch {
                break;
            }
        }
        initialiser.push_str(&token.to_string());
    }
    initialiser
}

fn num_chars_in(tokens: &mut IntoIter, ch: char) -> usize {
    tokens
        .clone()
        .filter(|token| {
            if let TokenTree::Punct(token) = token.clone() {
                if token.as_char() == ch {
                    return true;
                }
            }
            false
        })
        .count()
}

fn extract_group(token: &mut IntoIter, error_message: &str) -> IntoIter {
    match token.next().expect(error_message) {
        TokenTree::Group(group) => group,
        _ => panic!(error_message.to_owned()),
    }
    .stream()
    .into_iter()
}

enum Keyword {
    Inputs,
    Outputs,
    Processors,
    Connections,
}

impl Keyword {
    pub fn parse(token: TokenTree) -> Result<Self, ()> {
        match token.to_string().as_str() {
            "inputs" => Ok(Keyword::Inputs),
            "outputs" => Ok(Keyword::Outputs),
            "processors" => Ok(Keyword::Processors),
            "connections" => Ok(Keyword::Connections),
            _ => Err(()),
        }
    }
}

fn parse_group(tokens: &mut IntoIter) -> (Keyword, IntoIter) {
    let keyword = tokens
        .next()
        .expect("The graph declaration must start with a list of endpoints delimited by the 'endpoints' keyword");

    let _colon = tokens
        .next()
        .expect("Expected ':' character after 'endpoints' keyword");

    let missing_endpoint_group_message = "Expected a group of declaration delimited by braces '{}'";

    (
        Keyword::parse(keyword).expect("Unexpected keyword"),
        extract_group(tokens.by_ref(), missing_endpoint_group_message),
    )
}

trait ParsableDecl: Default {
    fn new(tokens: &mut IntoIter) -> Self {
        let mut decl = Self::default();
        decl.parse(tokens);
        decl
    }

    fn parse(&mut self, tokens: &mut IntoIter);
}

#[derive(Debug, Default)]
struct ProcessorDecl {
    name: String,
    initialiser: String,
}

#[derive(Debug, Default)]
struct ProcessorsDecl {
    decls: Vec<ProcessorDecl>,
}

impl ProcessorsDecl {
    pub fn to_processors_init(&self) -> String {
        let mut processor_init = String::new();

        for decl in &self.decls {
            processor_init.push_str(&format!(
                "\tlet {} = rume::make_processor({});\n",
                decl.name, decl.initialiser
            ));
        }

        processor_init
    }
}

impl ParsableDecl for ProcessorsDecl {
    fn parse(&mut self, tokens: &mut IntoIter) {
        let num_processors = num_chars_in(tokens.by_ref(), ',');

        for _ in 0..num_processors {
            let name = tokens
                .by_ref()
                .next()
                .expect("Expected a name for this processor")
                .to_string();

            let _semi_colon = tokens
                .by_ref()
                .next()
                .expect("Expected ':' character after an processor name");

            let initialiser = consume_until_char(tokens.by_ref(), ',');

            self.decls.push(ProcessorDecl { name, initialiser });
        }
    }
}

#[derive(Debug, Default)]
struct ConnectionDecl {
    tx_processor: String,
    tx_port: String,
    rx_processor: String,
    rx_port: String,
}

#[derive(Debug, Default)]
struct ConnectionsDecl {
    decls: Vec<ConnectionDecl>,
}

impl ParsableDecl for ConnectionsDecl {
    fn parse(&mut self, tokens: &mut IntoIter) {
        let num_connections = num_chars_in(tokens.by_ref(), '>');

        for _ in 0..num_connections {
            let tx_processor = tokens
                .by_ref()
                .next()
                .expect("Expected a name for this processor")
                .to_string();

            let tx_port = consume_until_char(tokens.by_ref(), '-');

            let _arrow_second_char = tokens
                .by_ref()
                .next()
                .expect("Expected '>' after '-' to form '->'");

            let rx_processor = tokens
                .by_ref()
                .next()
                .expect("Expected a name for this processor")
                .to_string();

            let rx_port = consume_until_char(tokens.by_ref(), ',');

            self.decls.push(ConnectionDecl {
                tx_processor,
                tx_port,
                rx_processor,
                rx_port,
            })
        }
    }
}

#[derive(Debug)]
enum GraphInputOption {
    Kind(InputEndpointKind),
    Range(Range<f32>),
    Init(f32),
    Smooth(u32),
    None,
}

impl Default for GraphInputOption {
    fn default() -> Self {
        Self::None
    }
}

impl GraphInputOption {
    pub fn kind(&mut self, tokens: &mut IntoIter) {
        let kind = tokens
            .by_ref()
            .next()
            .expect("Expected a value after an input endpoint init options");

        match kind.to_string().as_str() {
            "follow" => *self = GraphInputOption::Kind(InputEndpointKind::Follow),
            "trigger" => *self = GraphInputOption::Kind(InputEndpointKind::Trigger),
            _ => panic!(
                "Unsupported input endpoint kind: {}",
                kind.to_string().as_str()
            ),
        }
    }

    pub fn range(&mut self, tokens: &mut IntoIter) {
        let min_value = tokens
            .by_ref()
            .next()
            .expect("Expected a value after an input endpoint init options")
            .to_string()
            .parse::<f32>()
            .expect("Failed to parse input endpoint init value");

        let _dot = tokens
            .by_ref()
            .next()
            .expect("Expected a value after a '.' after a range's minimum value");

        let _dot = tokens
            .by_ref()
            .next()
            .expect("Expected a value after a '.' after a range's minimum value");

        let max_value = tokens
            .by_ref()
            .next()
            .expect("Expected a value after an input endpoint init options")
            .to_string()
            .parse::<f32>()
            .expect("Failed to parse input endpoint init value");

        *self = GraphInputOption::Range(min_value..max_value)
    }

    pub fn init(&mut self, tokens: &mut IntoIter) {
        let value = tokens
            .by_ref()
            .next()
            .expect("Expected a value after an input endpoint init options")
            .to_string()
            .parse::<f32>()
            .expect("Failed to parse input endpoint init value");

        *self = GraphInputOption::Init(value)
    }

    pub fn smooth(&mut self, tokens: &mut IntoIter) {
        let value = tokens
            .by_ref()
            .next()
            .expect("Expected a value after an input endpoint init options")
            .to_string()
            .parse::<u32>()
            .expect("Failed to parse input endpoint init value");

        *self = GraphInputOption::Smooth(value)
    }

    pub fn to_init(&self) -> String {
        match self {
            GraphInputOption::Kind(kind) => format!(".kind(rume::InputEndpointKind::{:#?})", kind),
            GraphInputOption::Range(range) => {
                format!(".range({:.32}..{:.32})", range.start, range.end)
            }
            GraphInputOption::Init(init) => format!(".init({:.32})", init),
            GraphInputOption::Smooth(smooth) => format!(".smooth({:})", smooth),
            GraphInputOption::None => String::new(),
        }
    }
}

impl ParsableDecl for GraphInputOption {
    fn parse(&mut self, tokens: &mut IntoIter) {
        let name = tokens
            .by_ref()
            .next()
            .expect("Expected a name for this endpoint")
            .to_string();

        let _colon = tokens
            .by_ref()
            .next()
            .expect("Expected ':' character after an endpoint name");

        match name.as_str() {
            "kind" => self.kind(tokens.by_ref()),
            "range" => self.range(tokens.by_ref()),
            "init" => self.init(tokens.by_ref()),
            "smooth" => self.smooth(tokens.by_ref()),
            _ => panic!("Unsupported input endpoint options: {}", name.as_str()),
        }
    }
}

#[derive(Debug, Default)]
struct GraphInputOptions {
    inner: Vec<GraphInputOption>,
}

impl ParsableDecl for GraphInputOptions {
    fn parse(&mut self, tokens: &mut IntoIter) {
        loop {
            self.inner.push(GraphInputOption::new(tokens));
            if tokens.by_ref().next().is_none() {
                break;
            }
        }
    }
}

#[derive(Debug, Default)]
struct GraphInput {
    name: String,
    options: Option<GraphInputOptions>,
}

impl GraphInput {
    pub fn new(name: String) -> Self {
        Self {
            name,
            options: None,
        }
    }

    pub fn to_endpoint_init(&self) -> String {
        let mut endpoint_init = String::new();

        endpoint_init.push_str(&format!(
            "let {} = rume::make_processor(rume::InputEndpointBuilder::new({}_consumer)\n",
            self.name, self.name
        ));

        if self.options.is_some() {
            for option in &self.options.as_ref().unwrap().inner {
                endpoint_init.push_str(&format!("\t\t{}\n", option.to_init()));
            }
        }

        endpoint_init.push_str("\t\t.build());");
        endpoint_init
    }
}

#[derive(Debug, Default)]
struct GraphInputs {
    decls: Vec<GraphInput>,
}

impl GraphInputs {
    pub fn to_endpoints_init(&self) -> String {
        let mut endpoints_init = String::new();

        for decl in &self.decls {
            endpoints_init.push_str(&format!(
                "\t{}\n\t{}\n\n",
                format!(
                    "let ({}_producer, {}_consumer) = rume::make_input_endpoint();",
                    decl.name, decl.name
                ),
                decl.to_endpoint_init()
            ));
        }

        endpoints_init
    }

    pub fn to_struct_decl(&self) -> String {
        let mut struct_decl = String::new();
        struct_decl.push_str("\npub struct Inputs {\n");

        for decl in &self.decls {
            struct_decl.push_str(&format!(
                "\tpub {}: rume::InputStreamProducer,\n",
                decl.name
            ));
        }

        struct_decl.push_str("}\n");
        struct_decl
    }

    pub fn to_struct_init(&self) -> String {
        let mut struct_init = String::new();
        struct_init.push_str("Inputs {");

        for decl in &self.decls {
            struct_init.push_str(&format!("{}: {}_producer, ", decl.name, decl.name));
        }

        struct_init.push('}');
        struct_init
    }
}

impl ParsableDecl for GraphInputs {
    fn parse(&mut self, tokens: &mut IntoIter) {
        let num_endpoints = num_chars_in(tokens.by_ref(), ',');
        for _ in 0..num_endpoints {
            let name = tokens
                .by_ref()
                .next()
                .expect("Expected a name for this endpoint")
                .to_string();

            let mut input = GraphInput::new(name);

            let character = tokens
                .by_ref()
                .next()
                .expect("Expected ',' character after an endpoint name");

            if character.to_string() == ":" {
                let mut raw_opts =
                    extract_group(tokens, "expected a group of input endpoint options");
                input.options = Some(GraphInputOptions::new(raw_opts.by_ref()));
            }

            self.decls.push(input);

            let _comma = tokens.by_ref().next();
        }
    }
}

#[derive(Debug, Default)]
struct GraphOutputs {
    names: Vec<String>,
}

impl GraphOutputs {
    pub fn to_endpoints_init(&self) -> String {
        let mut endpoints_init = String::new();

        for name in &self.names {
            endpoints_init.push_str(&format!(
                "\t{}\n\t{}\n\n",
                format!(
                    "let ({}_producer, {}_consumer) = rume::make_output_endpoint();",
                    name, name
                ),
                format!(
                    "let {} = rume::make_processor(rume::OutputEndpoint::new({}_producer));",
                    name, name
                )
            ));
        }

        endpoints_init
    }

    pub fn to_struct_decl(&self) -> String {
        let mut struct_decl = String::new();
        struct_decl.push_str("\npub struct Outputs {\n");

        for name in &self.names {
            struct_decl.push_str(&format!("\tpub {}: rume::OutputStreamConsumer,\n", name));
        }

        struct_decl.push_str("}\n");
        struct_decl
    }

    pub fn to_struct_init(&self) -> String {
        let mut struct_init = String::new();
        struct_init.push_str("Outputs {");

        for name in &self.names {
            struct_init.push_str(&format!("{}: {}_consumer, ", name, name));
        }

        struct_init.push('}');
        struct_init
    }
}

impl ParsableDecl for GraphOutputs {
    fn parse(&mut self, tokens: &mut IntoIter) {
        let num_endpoints = num_chars_in(tokens.by_ref(), ',');

        for _ in 0..num_endpoints {
            let name = tokens
                .by_ref()
                .next()
                .expect("Expected a name for this output")
                .to_string();

            self.names.push(name);

            let _comma = tokens
                .by_ref()
                .next()
                .expect("Expected ',' character after an output name");
        }
    }
}

#[derive(Debug, Default)]
struct GraphDecl {
    inputs: GraphInputs,
    outputs: GraphOutputs,
    processors: ProcessorsDecl,
    connections: ConnectionsDecl,
}

impl ParsableDecl for GraphDecl {
    fn parse(&mut self, mut tokens: &mut IntoIter) {
        loop {
            match parse_group(&mut tokens) {
                (Keyword::Inputs, mut group) => self.inputs = GraphInputs::new(&mut group),
                (Keyword::Outputs, mut group) => self.outputs = GraphOutputs::new(&mut group),
                (Keyword::Processors, mut group) => {
                    self.processors = ProcessorsDecl::new(&mut group)
                }
                (Keyword::Connections, mut group) => {
                    self.connections = ConnectionsDecl::new(&mut group)
                }
            }
            if tokens.by_ref().next().is_none() {
                break;
            }
        }
    }
}

impl ToString for GraphDecl {
    fn to_string(&self) -> String {
        let mut build_graph_fn = String::new();

        build_graph_fn.push_str("pub fn build() -> (rume::SignalChain, Inputs, Outputs) {\n");

        build_graph_fn.push_str(&self.inputs.to_endpoints_init());
        build_graph_fn.push_str(&self.outputs.to_endpoints_init());
        build_graph_fn.push_str(&self.processors.to_processors_init());

        build_graph_fn.push_str("\n\tlet chain = rume::SignalChainBuilder::default()\n");

        for decl in &self.inputs.decls {
            build_graph_fn.push_str(&format!("\t\t.processor({}.clone())\n", decl.name));
        }

        for name in &self.outputs.names {
            build_graph_fn.push_str(&format!("\t\t.processor({}.clone())\n", name));
        }

        for decl in &self.processors.decls {
            build_graph_fn.push_str(&format!("\t\t.processor({}.clone())\n", decl.name));
        }

        for decl in &self.connections.decls {
            build_graph_fn.push_str(&format!(
                "\t\t.connection(
                    \trume::OutputPort {{ proc: {}.clone(), port: Box::new({}.clone().borrow(){}.clone()) }},
                    \trume::InputPort {{ proc: {}.clone(), port: Box::new({}.clone().borrow(){}.clone()) }}
                )\n",
                decl.tx_processor,
                decl.tx_processor,
                decl.tx_port,
                decl.rx_processor,
                decl.rx_processor,
                decl.rx_port,
            ));
        }

        build_graph_fn.push_str("\t\t.build();\n\n");
        build_graph_fn.push_str(&format!(
            "\t( chain, {}, {} )",
            self.inputs.to_struct_init(),
            self.outputs.to_struct_init(),
        ));
        build_graph_fn.push('\n');
        build_graph_fn.push('}');
        build_graph_fn.push('\n');

        let input_struct_decl = format!("\t{}\n", self.inputs.to_struct_decl());
        let output_struct_decl = format!("\t{}\n", self.outputs.to_struct_decl());

        format!(
            "{}\n{}\n{}\n",
            input_struct_decl, output_struct_decl, build_graph_fn
        )
    }
}

pub fn graph(input: TokenStream) -> TokenStream {
    let mut tokens = input.into_iter();
    let graph = GraphDecl::new(&mut tokens);
    graph.to_string().parse().unwrap()
}
