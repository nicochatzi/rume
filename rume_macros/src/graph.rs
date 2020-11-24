use proc_macro::{token_stream::IntoIter, TokenStream, TokenTree};

fn consume_until_char(tokens: &mut IntoIter, ch: char) -> String {
    let mut initialiser = String::new();
    loop {
        let token = tokens
            .by_ref()
            .nth(0)
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
    match token.nth(0).expect(error_message) {
        TokenTree::Group(group) => group,
        _ => panic!(error_message.to_owned()),
    }
    .stream()
    .into_iter()
}

pub fn graph(input: TokenStream) -> TokenStream {
    let mut tokens = input.into_iter();

    //
    //  Construct list of endpoints.
    //
    //  ```
    //      endpoints: {
    //          audio_out: OutputEndpoint::new(audio_out_producer),
    //      },
    //  ```
    //
    let _endpoints_keyword = tokens
        .nth(0)
        .expect("The graph declaration must start with a list of endpoints delimited by the 'endpoints' keyword");

    let _colon = tokens
        .nth(0)
        .expect("Expected ':' character after 'endpoints' keyword");

    let missing_endpoint_group_message =
        "Expected a group of tokens delimited by braces '{}' after 'endpoints:'";

    let mut endpoint_tokens = extract_group(tokens.by_ref(), missing_endpoint_group_message);
    let mut endpoint_names = Vec::<String>::new();
    let mut endpoint_initialisers = Vec::<String>::new();
    let num_endpoints = num_chars_in(endpoint_tokens.by_ref(), ',');

    for _ in 0..num_endpoints {
        let endpoint_name = endpoint_tokens
            .by_ref()
            .nth(0)
            .expect("Expected a name for this endpoint");
        endpoint_names.push(endpoint_name.to_string());

        let _colon = endpoint_tokens
            .by_ref()
            .nth(0)
            .expect("Expected ':' character after 'endpoints' keyword");

        endpoint_initialisers.push(consume_until_char(endpoint_tokens.by_ref(), ','));
    }

    let _comma = tokens
        .by_ref()
        .nth(0)
        .expect("Expected ',' character after 'endpoints'");

    //
    //  Construct list of processors.
    //
    //  ```
    //      processors: {
    //          sine: Sine::default(),
    //          tanh: Tanh::default(),
    //      }
    //  ```
    //

    let _processor_keyword = tokens
        .nth(0)
        .expect("The graph declaration must have with a list of processors delimited by the 'processors' keyword after the endpoints declaration");

    let _colon = tokens
        .nth(0)
        .expect("Expected ':' character after 'processors' keyword");

    let missing_processor_group_message =
        "Expected a group of tokens delimited by braces '{}' after 'processors:'";

    let mut processor_tokens = extract_group(tokens.by_ref(), missing_processor_group_message);
    let mut processor_names = Vec::<String>::new();
    let mut processor_initialisers = Vec::<String>::new();
    let num_processors = num_chars_in(processor_tokens.by_ref(), ',');

    for _ in 0..num_processors {
        let processor_name = processor_tokens
            .by_ref()
            .nth(0)
            .expect("Expected a name for this processor");
        processor_names.push(processor_name.to_string());

        let _semi_colon = processor_tokens
            .by_ref()
            .nth(0)
            .expect("Expected ':' character after an processor name");

        processor_initialisers.push(consume_until_char(processor_tokens.by_ref(), ','));
    }

    let _comma = tokens
        .by_ref()
        .nth(0)
        .expect("Expected ',' character after 'processors'");

    //
    //   Construct list of connections.
    //
    //  ```
    //      connections: {
    //          sine.output -> tanh.input,
    //          tanh.output -> audio_out,
    //      }
    //  ```

    let _connections_keyword = tokens
        .nth(0)
        .expect("The graph declaration must have with a list of connections delimited by the 'connections' keyword after the processors declaration");

    let _colon = tokens
        .nth(0)
        .expect("Expected ':' character after 'connections' keyword");

    let missing_connection_group_message =
        "Expected a group of tokens delimited by braces '{}' after 'connections:'";

    let mut connection_tokens = extract_group(tokens.by_ref(), missing_connection_group_message);
    let mut tx_processors = Vec::<String>::new();
    let mut tx_ports = Vec::<String>::new();
    let mut rx_processors = Vec::<String>::new();
    let mut rx_ports = Vec::<String>::new();
    let num_connections = num_chars_in(connection_tokens.by_ref(), '>');

    for _ in 0..num_connections {
        let tx_processor = connection_tokens
            .by_ref()
            .nth(0)
            .expect("Expected a name for this processor");
        tx_processors.push(tx_processor.to_string());

        tx_ports.push(consume_until_char(connection_tokens.by_ref(), '-'));

        let _arrow_second_char = connection_tokens
            .by_ref()
            .nth(0)
            .expect("Expected '>' after '-' to form '->'");

        let rx_processor = connection_tokens
            .by_ref()
            .nth(0)
            .expect("Expected a name for this processor");
        rx_processors.push(rx_processor.to_string());

        rx_ports.push(consume_until_char(connection_tokens.by_ref(), ','));
    }

    //
    // Construct SignalChain object.
    //
    let mut graph_as_string = String::new();

    graph_as_string.push('{');
    graph_as_string.push('\n');
    graph_as_string.push_str("use std::{cell::RefCell, rc::Rc};");

    for (i, name) in endpoint_names.iter().enumerate() {
        graph_as_string.push_str(&format!(
            "\tlet {} = Rc::new(RefCell::new({}));\n",
            name, endpoint_initialisers[i]
        ));
    }

    for (i, name) in processor_names.iter().enumerate() {
        graph_as_string.push_str(&format!(
            "\tlet {} = Rc::new(RefCell::new({}));\n",
            name, processor_initialisers[i]
        ));
    }

    graph_as_string.push_str("\trume::SignalChain::new()\n");

    for name in endpoint_names {
        graph_as_string.push_str(&format!("\t\t.processor({}.clone())\n", name));
    }

    for name in processor_names {
        graph_as_string.push_str(&format!("\t\t.processor({}.clone())\n", name));
    }

    for i in 0..tx_ports.len() {
        graph_as_string.push_str(&format!(
            "\t\t.connection(
                \trume::OutputPort {{ proc: {}.clone(), port: Box::new({}.clone().borrow(){}.clone()) }},
                \trume::InputPort {{ proc: {}.clone(), port: Box::new({}.clone().borrow(){}.clone()) }},
            \t)\n",
            tx_processors[i],
            tx_processors[i],
            tx_ports[i],
            rx_processors[i],
            rx_processors[i],
            rx_ports[i],
        ));
    }

    graph_as_string.push_str("\t\t.build()\n");
    graph_as_string.push('}');
    graph_as_string.push('\n');

    // println!("{}", graph_as_string);

    graph_as_string.parse().unwrap()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
