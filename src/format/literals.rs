//! Formatting for literals nodes.

use ruby_prism::{
    BackReferenceReadNode, ClassVariableReadNode, ConstantPathNode, ConstantReadNode, FalseNode, FloatNode,
    GlobalVariableReadNode, ImaginaryNode, InstanceVariableReadNode, IntegerNode, ItLocalVariableReadNode,
    LocalVariableReadNode, NilNode, NumberedReferenceReadNode, RangeNode, RationalNode, SelfNode,
    ShareableConstantNode, SourceEncodingNode, SourceFileNode, SourceLineNode, TrueNode,
};

use super::Formatter;

pub fn back_reference_read_node(f: &mut Formatter<'_>, node: &BackReferenceReadNode<'_>) {
    f.text_of(&node.location());
}

pub fn class_variable_read_node(f: &mut Formatter<'_>, node: &ClassVariableReadNode<'_>) {
    f.text_of(&node.location());
}

pub fn constant_path_node(f: &mut Formatter<'_>, node: &ConstantPathNode<'_>) {
    if let Some(parent) = node.parent() {
        f.node(&parent);
    }
    f.b.text("::");
    f.text_of(&node.name_loc());
}

pub fn constant_read_node(f: &mut Formatter<'_>, node: &ConstantReadNode<'_>) {
    f.text_of(&node.location());
}

pub fn false_node(f: &mut Formatter<'_>, node: &FalseNode<'_>) {
    f.text_of(&node.location());
}

pub fn float_node(f: &mut Formatter<'_>, node: &FloatNode<'_>) {
    f.text_of(&node.location());
}

pub fn global_variable_read_node(f: &mut Formatter<'_>, node: &GlobalVariableReadNode<'_>) {
    f.text_of(&node.location());
}

pub fn imaginary_node(f: &mut Formatter<'_>, node: &ImaginaryNode<'_>) {
    f.text_of(&node.location());
}

pub fn instance_variable_read_node(f: &mut Formatter<'_>, node: &InstanceVariableReadNode<'_>) {
    f.text_of(&node.location());
}

/// Plain decimal integers of five or more digits get `_` thousands
/// separators; anything with a prefix, a leading zero or its own
/// underscores is kept as written.
pub fn integer_node(f: &mut Formatter<'_>, node: &IntegerNode<'_>) {
    if !f.options.normalize_number_separators {
        f.text_of(&node.location());
        return;
    }
    let text = f.slice(&node.location());
    let (sign, digits) = match text.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", text),
    };
    let plain = digits.len() >= 5 && !digits.starts_with('0') && digits.bytes().all(|b| b.is_ascii_digit());
    if !plain {
        f.text_of(&node.location());
        return;
    }
    let mut out = String::with_capacity(text.len() + digits.len() / 3);
    out.push_str(sign);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push('_');
        }
        out.push(c);
    }
    f.b.text(out);
}

pub fn it_local_variable_read_node(f: &mut Formatter<'_>, node: &ItLocalVariableReadNode<'_>) {
    f.text_of(&node.location());
}

pub fn local_variable_read_node(f: &mut Formatter<'_>, node: &LocalVariableReadNode<'_>) {
    f.text_of(&node.location());
}

pub fn nil_node(f: &mut Formatter<'_>, node: &NilNode<'_>) {
    f.text_of(&node.location());
}

pub fn numbered_reference_read_node(f: &mut Formatter<'_>, node: &NumberedReferenceReadNode<'_>) {
    f.text_of(&node.location());
}

pub fn range_node(f: &mut Formatter<'_>, node: &RangeNode<'_>) {
    if let Some(left) = node.left() {
        f.node(&left);
    }
    f.text_of(&node.operator_loc());
    if let Some(right) = node.right() {
        f.node(&right);
    }
}

pub fn rational_node(f: &mut Formatter<'_>, node: &RationalNode<'_>) {
    f.text_of(&node.location());
}

pub fn self_node(f: &mut Formatter<'_>, node: &SelfNode<'_>) {
    f.text_of(&node.location());
}

/// The `# shareable_constant_value` magic comment is an ordinary comment;
/// only the wrapped assignment prints.
pub fn shareable_constant_node(f: &mut Formatter<'_>, node: &ShareableConstantNode<'_>) {
    f.node(&node.write());
}

pub fn source_encoding_node(f: &mut Formatter<'_>, node: &SourceEncodingNode<'_>) {
    f.text_of(&node.location());
}

pub fn source_file_node(f: &mut Formatter<'_>, node: &SourceFileNode<'_>) {
    f.text_of(&node.location());
}

pub fn source_line_node(f: &mut Formatter<'_>, node: &SourceLineNode<'_>) {
    f.text_of(&node.location());
}

pub fn true_node(f: &mut Formatter<'_>, node: &TrueNode<'_>) {
    f.text_of(&node.location());
}
