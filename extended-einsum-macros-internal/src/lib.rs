use std::iter;

use proc_macro2::*;
use quote::*;

use itertools::Itertools;

use syn::parse::*;
use syn::punctuated::Punctuated;
use syn::*;

use ndarray as nd;

use itertools;
use std::fmt::Display;
use syn::token::Token;

// struct TaggedArray {}

// Example:
//
// register_dims! { D1, D2, D3<V1, V2, V3, V4> };

// macro_rules! register_dims {
//     () => {

//     };
// }

struct Term {
    array_name: Ident,
    _paren_token: token::Bracket,
    paren_fields: Punctuated<Ident, Token![,]>,
}

impl Display for Term {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        // TODO add the axes

        write!(f, "{}[TODO]", self.array_name)
    }
}

impl Parse for Term {
    fn parse(stream: ParseStream) -> Result<Self> {
        let content;
        Ok(Term {
            array_name: stream.parse()?,
            _paren_token: bracketed!(content in stream),
            paren_fields: content
                .parse_terminated(Ident::parse)?,
        })
    }
}

struct ShapeSpec {
    _paren_token: token::Bracket,
    paren_fields: Punctuated<Ident, Token![,]>,
}

impl Parse for ShapeSpec {
    fn parse(stream: ParseStream) -> Result<Self> {
        let content;

        Ok(ShapeSpec {
            _paren_token: bracketed!(content in stream),
            paren_fields: content
                .parse_terminated(Ident::parse)?,
        })
    }
}

enum Node {
    Op(Op),
    Term(Term),
}

impl Parse for Node {
    fn parse(stream: ParseStream) -> Result<Self> {
        // TODO check https://docs.rs/syn/1.0.77/syn/parse/index.html

        let first_term: Term = stream.parse()?;

        if stream.fork().parse::<syn::BinOp>().is_ok() {
            Ok(Node::Op(Op {
                // TODO implement a better parser logic here
                lhs: Box::new(Node::Term(first_term)),
                op: stream.parse()?,
                rhs: Box::new(stream.parse()?),
            }))
        } else {
            Ok(Node::Term(first_term))
        }
    }
}

struct Op {
    lhs: Box<Node>,
    op: syn::BinOp,
    rhs: Box<Node>,
}

impl<'a> IntoIterator for &'a Node {
    type Item = &'a Term;
    type IntoIter = std::vec::IntoIter<&'a Term>;

    fn into_iter(self) -> Self::IntoIter {
        fn append<'a>(
            tree: &'a Node,
            v: &mut Vec<&'a Term>,
        ) {
            match tree {
                Node::Term(term) => {
                    v.push(&term);
                }
                Node::Op(op) => {
                    append(&op.lhs, v);
                    append(&op.rhs, v);
                }
            }
        }

        let mut result = vec![];

        append(self, &mut result);

        result.into_iter()
    }
}

impl Node {
    fn render_template(
        &self,
        removed_dims: &Vec<&Ident>,
    ) -> proc_macro2::TokenStream {
        match self {
            Node::Op(op) => {
                let op_token = op.op;

                let lhs_tokens =
                    op.lhs.render_template(removed_dims);
                let rhs_tokens =
                    op.rhs.render_template(removed_dims);

                quote! {
                    #lhs_tokens #op_token #rhs_tokens
                }
            }

            Node::Term(term) => {
                let array_name = &term.array_name;

                let paren_fields = term
                    .paren_fields
                    .iter()
                    .map(|x| format_ident!("{}_index", x));

                let mut divider = quote! { 1.0 };

                for removed_dim in removed_dims.iter() {
                    if !term
                        .paren_fields
                        .iter()
                        .contains(removed_dim)
                    {
                        let len = format_ident!(
                            "{}_length",
                            removed_dim
                        );

                        divider = quote! { #divider * (#len as f32) };
                    }
                }

                quote! {(#array_name[[#(#paren_fields),*]] / (#divider))}
            }
        }
    }
}

struct Line {
    root_expression_node: Node,
    _rarrow_token: Token!(~),
    shape_spec: ShapeSpec,
}

impl Parse for Line {
    fn parse(stream: ParseStream) -> Result<Self> {
        Ok(Line {
            root_expression_node: stream.parse()?,
            _rarrow_token: stream.parse()?,
            shape_spec: stream.parse()?,
        })
    }
}

struct FullExpr {
    fields: Punctuated<Line, Token![;]>,
}

impl Parse for FullExpr {
    fn parse(stream: ParseStream) -> Result<Self> {
        Ok(FullExpr {
            fields: stream.parse_terminated(Line::parse)?,
        })
    }
}

// ein!{ a[D1, D2] * b[D2, D3] + c[D3] ~ [D1, D3] };

// steps:
//
// * get a list of all the variables
// * allocate the output expression
// * create loops for the output expression
// * create loops for variables not in output expression
// *

fn process_ein_line(
    line: &Line,
) -> proc_macro2::TokenStream {
    let all_axis_symbols: Vec<&Ident> = line
        .root_expression_node
        .into_iter()
        .map(|x| x.paren_fields.iter())
        .flatten()
        .unique()
        .collect();

    let output_dims: Vec<&Ident> =
        line.shape_spec.paren_fields.iter().collect();

    let removed_dims: Vec<&Ident> = all_axis_symbols
        .iter()
        .filter(|&x| !output_dims.contains(x))
        .cloned()
        .collect();

    let mut array_bounds_checks = Vec::new();

    for axis_symbol in all_axis_symbols.iter() {
        struct AxisInTerm<'a> {
            term: &'a Term,
            axis_symbol: &'a Ident,
            axis_index_in_term: usize,
        }

        // Step 1) Collect all instances of "axis_symbol" in the expression

        let all_matching_axes_in_terms: Vec<AxisInTerm> =
            line.root_expression_node
                .into_iter()
                .map(|term| {
                    let term = term.clone();

                    let axes_in_term: Vec<AxisInTerm> =
                        term.paren_fields
                            .iter()
                            .enumerate()
                            .filter(|(_, s)| {
                                **s == **axis_symbol
                            })
                            .map(|(i, s)| AxisInTerm {
                                term: term.clone(),
                                axis_symbol: s,
                                axis_index_in_term: i,
                            })
                            .collect();

                    axes_in_term
                })
                .flatten()
                .collect();

        assert_ne!(all_matching_axes_in_terms.len(), 0);

        // Step 2) Now we need to construct a "for" loop for this axis. It is
        // assumed that all the matching axes across all the arrays have the
        // same length, here we construct a runtime check.
        //
        // TODO: Do a compile time check if using fixed length arrays

        let array_name_ref =
            &all_matching_axes_in_terms[0].term.array_name;

        let axis_index_in_term_ref: usize =
            all_matching_axes_in_terms[0]
                .axis_index_in_term;

        let axis_length_variable =
            format_ident!("{}_length", axis_symbol);

        array_bounds_checks.push(quote! {
            let #axis_length_variable = #array_name_ref.shape()[#axis_index_in_term_ref];
        });

        for matching_axis in
            all_matching_axes_in_terms.iter().skip(1)
        {
            let array_name = &matching_axis.term.array_name;

            let array_index: usize =
                matching_axis.axis_index_in_term;

            array_bounds_checks.push(quote! {
                assert_eq!(#array_name.shape()[#array_index], #axis_length_variable);
            });
        }
    }

    // Step 3: Allocate the output buffer

    let output_buffer_size_variables = output_dims
        .iter()
        .map(|x| format_ident!("{}_length", x));

    let output_buffer_allocation = quote! {
        let mut result = nd::Array::<f32, _>::zeros((#(#output_buffer_size_variables),*));
    };

    // Step 5: Render the output buffer

    let axes = line
        .shape_spec
        .paren_fields
        .iter()
        .map(|x| format_ident!("{}_index", x));

    // Step 6: Recursively render the equation using the loop variables

    let rendered_expression = line
        .root_expression_node
        .render_template(&removed_dims);

    let mut loop_contents = quote! {
        result[[#(#axes),*]] += #rendered_expression;
    };

    // Step 4: Add for loops for all the variables in the expression

    for axis_symbol in all_axis_symbols.iter() {
        let axis_index_variable =
            format_ident!("{}_index", axis_symbol);

        let axis_length_variable =
            format_ident!("{}_length", axis_symbol);

        loop_contents = quote! {
            for #axis_index_variable in 0..#axis_length_variable { #loop_contents }
        };
    }

    return quote! {
        #(#array_bounds_checks)*

        #output_buffer_allocation

        #loop_contents
    };
}

pub fn ein_internal(
    input: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    // Glossary:
    //
    // * Array: The name of the array, e.g. a
    // * Axes: The symbols introduced to index the array
    // * Term: Array + Axes
    // * Node: Forms a tree of terms + ops

    // let signature =
    // syn::parse_macro_input!(input as FullExpr);

    let signature: FullExpr = syn::parse2(input).unwrap();

    let mut all_lines =
        Vec::<proc_macro2::TokenStream>::new();

    for line in signature.fields {
        all_lines.push(process_ein_line(&line));
    }

    return quote! {
        {
            #(#all_lines)*

            result
        }
    };
}

#[cfg(test)]
mod tests {

    use ndarray as nd;
    use std::str::FromStr;

    use crate::*;

    #[test]
    fn basic_tests() {
        let ts =
            proc_macro2::TokenStream::from_str("a[D1, D2]")
                .unwrap();

        let ts = syn::parse2::<Term>(ts).unwrap();

        println!("{}", ts);

        // let ts = proc_macro2::TokenStream::from_str("a[D1, D2] * b[D2, D3]").unwrap();
        //
        // let ts = syn::parse2::<Op>(ts).unwrap();
        //
        // // println!("{}", ts);

        let ts = proc_macro2::TokenStream::from_str(
            "a[D1, D2] * b[D2, D3]",
        )
        .unwrap();

        let ts = syn::parse2::<Node>(ts).unwrap();

        // println!("{}", ts);

        let ts = proc_macro2::TokenStream::from_str(
            "a[D1, D2] * b[D2, D3] + c[D3]",
        )
        .unwrap();

        let ts = syn::parse2::<Node>(ts).unwrap();

        // println!("{}", ts);

        let ts = proc_macro2::TokenStream::from_str(
            "a[D1, D2] * b[D2, D3] + c[D3] ~ [D1, D3]",
        )
        .unwrap();

        let ts = syn::parse2::<Line>(ts).unwrap();

        // println!("{}", ts);

        let ts = proc_macro2::TokenStream::from_str("a[D1, D2] * b[D2, D3] + c[D3] ~ [D1, D3]; a[D1, D2] * b[D2, D3] + c[D3] ~ [D1, D3]").unwrap();

        let ts = syn::parse2::<FullExpr>(ts).unwrap();

        // println!("{}", ts);

        let ts = proc_macro2::TokenStream::from_str(
            "a[D1, D2] * b[D2, D3] + c[D3] ~ [D1, D3]",
        )
        .unwrap();

        let ts = ein_internal(ts);

        println!("{}", ts);
    }
}
