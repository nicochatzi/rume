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

#[derive(Debug, Default)]
struct GraphEndpoints {
    names: Vec<String>,
}

enum GraphEndpointsKind {
    Inputs,
    Outputs,
}

impl GraphEndpoints {
    pub fn to_struct_decl(&self, kind: GraphEndpointsKind) -> String {
        match kind {
            GraphEndpointsKind::Inputs => {
                self.to_named_struct_decl("Inputs", "rume::InputStreamProducer")
            }
            GraphEndpointsKind::Outputs => {
                self.to_named_struct_decl("Outputs", "rume::OutputStreamConsumer")
            }
        }
    }

    fn to_named_struct_decl(&self, struct_name: &str, field_type: &str) -> String {
        let mut struct_decl = String::new();
        struct_decl.push_str(&format!("pub struct {} {{\n", struct_name));

        for name in &self.names {
            struct_decl.push_str(&format!("\tpub {}: {},\n", name, field_type));
        }

        struct_decl.push_str("}\n");
        struct_decl
    }

    pub fn to_struct_init(&self, kind: GraphEndpointsKind) -> String {
        match kind {
            GraphEndpointsKind::Inputs => self.to_named_struct_init("Inputs", "producer"),
            GraphEndpointsKind::Outputs => self.to_named_struct_init("Outputs", "consumer"),
        }
    }

    fn to_named_struct_init(&self, struct_name: &str, suffix: &str) -> String {
        let mut struct_init = String::new();
        struct_init.push_str(&format!("{} {{", struct_name));

        for name in &self.names {
            struct_init.push_str(&format!("{}: {}_{}, ", name, name, suffix));
        }

        struct_init.push_str("}");
        struct_init
    }
}

impl ParsableDecl for GraphEndpoints {
    fn parse(&mut self, tokens: &mut IntoIter) {
        let num_endpoints = num_chars_in(tokens.by_ref(), ',');

        for _ in 0..num_endpoints {
            let name = tokens
                .by_ref()
                .next()
                .expect("Expected a name for this endpoint")
                .to_string();

            let _comma = tokens
                .by_ref()
                .next()
                .expect("Expected ',' character after an endpoint name");

            self.names.push(name);
        }
    }
}

#[derive(Debug, Default)]
struct GraphDecl {
    inputs: Option<GraphEndpoints>,
    outputs: Option<GraphEndpoints>,
    processors: Option<ProcessorsDecl>,
    connections: ConnectionsDecl,
}

impl ParsableDecl for GraphDecl {
    fn parse(&mut self, mut tokens: &mut IntoIter) {
        loop {
            match parse_group(&mut tokens) {
                (Keyword::Inputs, mut group) => self.inputs = Some(GraphEndpoints::new(&mut group)),
                (Keyword::Outputs, mut group) => {
                    self.outputs = Some(GraphEndpoints::new(&mut group))
                }
                (Keyword::Processors, mut group) => {
                    self.processors = Some(ProcessorsDecl::new(&mut group))
                }
                (Keyword::Connections, mut group) => {
                    self.connections = ConnectionsDecl::new(&mut group)
                }
            }
            if let None = tokens.by_ref().next() {
                break;
            }
        }
    }
}

impl ToString for GraphDecl {
    fn to_string(&self) -> String {
        let mut make_graph_fn = String::new();

        make_graph_fn.push_str("pub fn make() -> (rume::SignalChain, Inputs, Outputs) {\n");

        for name in &self.inputs.as_ref().unwrap().names {
            let endpoint = format!(
                "\tlet ({}_producer, {}_consumer) = rume::make_input_endpoint();\n",
                name, name
            );
            let processor = format!(
                "\tlet {} = rume::make_processor(rume::InputEndpoint::new({}_consumer));\n",
                name, name
            );

            make_graph_fn.push_str(&format!("\t{}\t\n{}\n", endpoint, processor));
        }

        for name in &self.outputs.as_ref().unwrap().names {
            let endpoint = format!(
                "\tlet ({}_producer, {}_consumer) = rume::make_output_endpoint();\n",
                name, name
            );
            let processor = format!(
                "\tlet {} = rume::make_processor(rume::OutputEndpoint::new({}_producer));\n",
                name, name
            );

            make_graph_fn.push_str(&format!("\t{}\t\n{}\n", endpoint, processor));
        }

        for decl in &self.processors.as_ref().unwrap().decls {
            make_graph_fn.push_str(&format!(
                "\tlet {} = rume::make_processor({});\n",
                decl.name, decl.initialiser
            ));
        }

        make_graph_fn.push_str("\tlet chain = rume::SignalChainBuilder::default()\n");

        for name in &self.inputs.as_ref().unwrap().names {
            make_graph_fn.push_str(&format!("\t\t.processor({}.clone())\n", name));
        }

        for name in &self.outputs.as_ref().unwrap().names {
            make_graph_fn.push_str(&format!("\t\t.processor({}.clone())\n", name));
        }

        for decl in &self.processors.as_ref().unwrap().decls {
            make_graph_fn.push_str(&format!("\t\t.processor({}.clone())\n", decl.name));
        }

        for decl in &self.connections.decls {
            make_graph_fn.push_str(&format!(
                "\t\t.connection(
                    \trume::OutputPort {{ proc: {}.clone(), port: Box::new({}.clone().borrow(){}.clone()) }},
                    \trume::InputPort {{ proc: {}.clone(), port: Box::new({}.clone().borrow(){}.clone()) }}
                \t)\n",
                decl.tx_processor,
                decl.tx_processor,
                decl.tx_port,
                decl.rx_processor,
                decl.rx_processor,
                decl.rx_port,
            ));
        }

        make_graph_fn.push_str("\t\t.build();\n");
        make_graph_fn.push_str(&format!(
            "\t( chain, {}, {} )",
            self.inputs
                .as_ref()
                .unwrap()
                .to_struct_init(GraphEndpointsKind::Inputs),
            self.outputs
                .as_ref()
                .unwrap()
                .to_struct_init(GraphEndpointsKind::Outputs),
        ));
        make_graph_fn.push('}');
        make_graph_fn.push('\n');

        let input_struct_decl = format!(
            "\t{}\n",
            self.inputs
                .as_ref()
                .unwrap()
                .to_struct_decl(GraphEndpointsKind::Inputs)
        );

        let output_struct_decl = format!(
            "\t{}\n",
            self.outputs
                .as_ref()
                .unwrap()
                .to_struct_decl(GraphEndpointsKind::Outputs)
        );

        format!(
            "{}\n{}\n{}\n",
            input_struct_decl, output_struct_decl, make_graph_fn
        )
    }
}

pub fn graph(input: TokenStream) -> TokenStream {
    let mut tokens = input.into_iter();
    let graph = GraphDecl::new(&mut tokens);
    println!("{}", graph.to_string());
    graph.to_string().parse().unwrap()
}
