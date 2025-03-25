use ruby_prism::*;

use crate::{parser_state::ConcreteParserState, util::loc_to_string};

pub fn format_node(ps: &mut dyn ConcreteParserState, node: Node) {
    match node {
        Node::AliasGlobalVariableNode { .. } => todo!(),
        Node::AliasMethodNode { .. } => todo!(),
        Node::AlternationPatternNode { .. } => todo!(),
        Node::AndNode { .. } => todo!(),
        Node::ArgumentsNode { .. } => todo!(),
        Node::ArrayNode { .. } => todo!(),
        Node::ArrayPatternNode { .. } => todo!(),
        Node::AssocNode { .. } => todo!(),
        Node::AssocSplatNode { .. } => todo!(),
        Node::BackReferenceReadNode { .. } => todo!(),
        Node::BeginNode { .. } => todo!(),
        Node::BlockArgumentNode { .. } => todo!(),
        Node::BlockLocalVariableNode { .. } => todo!(),
        Node::BlockNode { .. } => todo!(),
        Node::BlockParameterNode { .. } => todo!(),
        Node::BlockParametersNode { .. } => todo!(),
        Node::BreakNode { .. } => todo!(),
        Node::CallAndWriteNode { .. } => todo!(),
        Node::CallNode { .. } => todo!(),
        Node::CallOperatorWriteNode { .. } => todo!(),
        Node::CallOrWriteNode { .. } => todo!(),
        Node::CallTargetNode { .. } => todo!(),
        Node::CapturePatternNode { .. } => todo!(),
        Node::CaseMatchNode { .. } => todo!(),
        Node::CaseNode { .. } => todo!(),
        Node::ClassNode { .. } => todo!(),
        Node::ClassVariableAndWriteNode { .. } => todo!(),
        Node::ClassVariableOperatorWriteNode { .. } => todo!(),
        Node::ClassVariableOrWriteNode { .. } => todo!(),
        Node::ClassVariableReadNode { .. } => todo!(),
        Node::ClassVariableTargetNode { .. } => todo!(),
        Node::ClassVariableWriteNode { .. } => todo!(),
        Node::ConstantAndWriteNode { .. } => todo!(),
        Node::ConstantOperatorWriteNode { .. } => todo!(),
        Node::ConstantOrWriteNode { .. } => todo!(),
        Node::ConstantPathAndWriteNode { .. } => todo!(),
        Node::ConstantPathNode { .. } => todo!(),
        Node::ConstantPathOperatorWriteNode { .. } => todo!(),
        Node::ConstantPathOrWriteNode { .. } => todo!(),
        Node::ConstantPathTargetNode { .. } => todo!(),
        Node::ConstantPathWriteNode { .. } => todo!(),
        Node::ConstantReadNode { .. } => todo!(),
        Node::ConstantTargetNode { .. } => todo!(),
        Node::ConstantWriteNode { .. } => todo!(),
        Node::DefNode { .. } => todo!(),
        Node::DefinedNode { .. } => todo!(),
        Node::ElseNode { .. } => todo!(),
        Node::EmbeddedStatementsNode { .. } => todo!(),
        Node::EmbeddedVariableNode { .. } => todo!(),
        Node::EnsureNode { .. } => todo!(),
        Node::FalseNode { .. } => todo!(),
        Node::FindPatternNode { .. } => todo!(),
        Node::FlipFlopNode { .. } => todo!(),
        Node::FloatNode { .. } => format_float_node(ps, node.as_float_node().unwrap()),
        Node::ForNode { .. } => todo!(),
        Node::ForwardingArgumentsNode { .. } => todo!(),
        Node::ForwardingParameterNode { .. } => todo!(),
        Node::ForwardingSuperNode { .. } => todo!(),
        Node::GlobalVariableAndWriteNode { .. } => todo!(),
        Node::GlobalVariableOperatorWriteNode { .. } => todo!(),
        Node::GlobalVariableOrWriteNode { .. } => todo!(),
        Node::GlobalVariableReadNode { .. } => todo!(),
        Node::GlobalVariableTargetNode { .. } => todo!(),
        Node::GlobalVariableWriteNode { .. } => todo!(),
        Node::HashNode { .. } => todo!(),
        Node::HashPatternNode { .. } => todo!(),
        Node::IfNode { .. } => todo!(),
        Node::ImaginaryNode { .. } => todo!(),
        Node::ImplicitNode { .. } => todo!(),
        Node::ImplicitRestNode { .. } => todo!(),
        Node::InNode { .. } => todo!(),
        Node::IndexAndWriteNode { .. } => todo!(),
        Node::IndexOperatorWriteNode { .. } => todo!(),
        Node::IndexOrWriteNode { .. } => todo!(),
        Node::IndexTargetNode { .. } => todo!(),
        Node::InstanceVariableAndWriteNode { .. } => todo!(),
        Node::InstanceVariableOperatorWriteNode { .. } => todo!(),
        Node::InstanceVariableOrWriteNode { .. } => todo!(),
        Node::InstanceVariableReadNode { .. } => todo!(),
        Node::InstanceVariableTargetNode { .. } => todo!(),
        Node::InstanceVariableWriteNode { .. } => todo!(),
        Node::IntegerNode { .. } => format_integer_node(ps, node.as_integer_node().unwrap()),
        Node::InterpolatedMatchLastLineNode { .. } => todo!(),
        Node::InterpolatedRegularExpressionNode { .. } => todo!(),
        Node::InterpolatedStringNode { .. } => todo!(),
        Node::InterpolatedSymbolNode { .. } => todo!(),
        Node::InterpolatedXStringNode { .. } => todo!(),
        Node::ItLocalVariableReadNode { .. } => todo!(),
        Node::ItParametersNode { .. } => todo!(),
        Node::KeywordHashNode { .. } => todo!(),
        Node::KeywordRestParameterNode { .. } => todo!(),
        Node::LambdaNode { .. } => todo!(),
        Node::LocalVariableAndWriteNode { .. } => todo!(),
        Node::LocalVariableOperatorWriteNode { .. } => todo!(),
        Node::LocalVariableOrWriteNode { .. } => todo!(),
        Node::LocalVariableReadNode { .. } => todo!(),
        Node::LocalVariableTargetNode { .. } => todo!(),
        Node::LocalVariableWriteNode { .. } => todo!(),
        Node::MatchLastLineNode { .. } => todo!(),
        Node::MatchPredicateNode { .. } => todo!(),
        Node::MatchRequiredNode { .. } => todo!(),
        Node::MatchWriteNode { .. } => todo!(),
        Node::MissingNode { .. } => todo!(),
        Node::ModuleNode { .. } => todo!(),
        Node::MultiTargetNode { .. } => todo!(),
        Node::MultiWriteNode { .. } => todo!(),
        Node::NextNode { .. } => todo!(),
        Node::NilNode { .. } => todo!(),
        Node::NoKeywordsParameterNode { .. } => todo!(),
        Node::NumberedParametersNode { .. } => todo!(),
        Node::NumberedReferenceReadNode { .. } => todo!(),
        Node::OptionalKeywordParameterNode { .. } => todo!(),
        Node::OptionalParameterNode { .. } => todo!(),
        Node::OrNode { .. } => todo!(),
        Node::ParametersNode { .. } => todo!(),
        Node::ParenthesesNode { .. } => todo!(),
        Node::PinnedExpressionNode { .. } => todo!(),
        Node::PinnedVariableNode { .. } => todo!(),
        Node::PostExecutionNode { .. } => todo!(),
        Node::PreExecutionNode { .. } => todo!(),
        Node::ProgramNode { .. } => format_program(ps, node.as_program_node().unwrap()),
        Node::RangeNode { .. } => todo!(),
        Node::RationalNode { .. } => todo!(),
        Node::RedoNode { .. } => todo!(),
        Node::RegularExpressionNode { .. } => todo!(),
        Node::RequiredKeywordParameterNode { .. } => todo!(),
        Node::RequiredParameterNode { .. } => todo!(),
        Node::RescueModifierNode { .. } => todo!(),
        Node::RescueNode { .. } => todo!(),
        Node::RestParameterNode { .. } => todo!(),
        Node::RetryNode { .. } => todo!(),
        Node::ReturnNode { .. } => todo!(),
        Node::SelfNode { .. } => todo!(),
        Node::ShareableConstantNode { .. } => todo!(),
        Node::SingletonClassNode { .. } => todo!(),
        Node::SourceEncodingNode { .. } => todo!(),
        Node::SourceFileNode { .. } => todo!(),
        Node::SourceLineNode { .. } => todo!(),
        Node::SplatNode { .. } => todo!(),
        Node::StatementsNode { .. } => format_statements(ps, node.as_statements_node().unwrap()),
        Node::StringNode { .. } => todo!(),
        Node::SuperNode { .. } => todo!(),
        Node::SymbolNode { .. } => todo!(),
        Node::TrueNode { .. } => todo!(),
        Node::UndefNode { .. } => todo!(),
        Node::UnlessNode { .. } => todo!(),
        Node::UntilNode { .. } => todo!(),
        Node::WhenNode { .. } => todo!(),
        Node::WhileNode { .. } => todo!(),
        Node::XStringNode { .. } => todo!(),
        Node::YieldNode { .. } => todo!(),
    }
}

fn format_program(ps: &mut dyn ConcreteParserState, program_node: ProgramNode) {
    format_statements(ps, program_node.statements());
}

fn format_statements(ps: &mut dyn ConcreteParserState, statements_node: StatementsNode) {
    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            for node in statements_node.body().iter() {
                format_node(ps, node);
            }
        }),
    );
}

fn format_integer_node(ps: &mut dyn ConcreteParserState, integer_node: IntegerNode) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_at_offset(
        ps,
        loc_to_string(integer_node.location()),
        integer_node.location().start_offset(),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

fn format_float_node(ps: &mut dyn ConcreteParserState, float_node: FloatNode) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_at_offset(
        ps,
        loc_to_string(float_node.location()),
        float_node.location().start_offset(),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

fn handle_string_at_offset(ps: &mut dyn ConcreteParserState, ident: String, offset: usize) {
    ps.at_offset(offset);
    ps.emit_ident(ident);
}
