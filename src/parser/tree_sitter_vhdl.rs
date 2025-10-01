use tree_sitter::{Language, Parser, Tree, Node};

extern "C" {
    fn tree_sitter_vhdl() -> Language;
}

/// Tree-sitter VHDL language binding
pub fn language() -> Language {
    unsafe { tree_sitter_vhdl() }
}

/// Tree-sitter based VHDL parser
pub struct TreeSitterVHDLParser {
    parser: Parser,
}

impl TreeSitterVHDLParser {
    pub fn new() -> anyhow::Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&language())
            .map_err(|e| anyhow::anyhow!("Failed to set VHDL language: {}", e))?;
        
        Ok(Self { parser })
    }

    pub fn parse(&mut self, source: &str) -> anyhow::Result<Tree> {
        self.parser.parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse VHDL source"))
    }

    pub fn parse_with_old_tree(&mut self, source: &str, old_tree: Option<&Tree>) -> anyhow::Result<Tree> {
        self.parser.parse(source, old_tree)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse VHDL source"))
    }
}

/// Helper functions for traversing VHDL AST nodes
pub struct VHDLASTHelper;

impl VHDLASTHelper {
    /// Get the text content of a node
    pub fn node_text<'a>(node: &Node, source: &'a str) -> &'a str {
        &source[node.start_byte()..node.end_byte()]
    }

    /// Find child nodes by type
    pub fn find_children_by_type<'a>(node: &'a Node<'a>, node_type: &str) -> Vec<Node<'a>> {
        let mut children = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == node_type {
                children.push(child);
            }
        }
        
        children
    }

    /// Find first child node by type
    pub fn find_child_by_type<'a>(node: &'a Node<'a>, node_type: &str) -> Option<Node<'a>> {
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == node_type {
                return Some(child);
            }
        }
        
        None
    }

    /// Recursively find all nodes of a specific type
    pub fn find_all_nodes_by_type<'a>(node: &'a Node<'a>, node_type: &str) -> Vec<Node<'a>> {
        let mut nodes = Vec::new();
        
        // Check current node
        if node.kind() == node_type {
            nodes.push(*node);
        }
        
        // Recursively check children using a queue to avoid lifetime issues
        let mut queue = vec![*node];
        while let Some(current) = queue.pop() {
            let mut cursor = current.walk();
            for child in current.children(&mut cursor) {
                if child.kind() == node_type {
                    nodes.push(child);
                }
                queue.push(child);
            }
        }
        
        nodes
    }

    /// Get named children only (ignoring punctuation and keywords)
    pub fn get_named_children<'a>(node: &'a Node<'a>) -> Vec<Node<'a>> {
        let mut children = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.is_named() {
                children.push(child);
            }
        }
        
        children
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = TreeSitterVHDLParser::new();
        assert!(parser.is_ok());
    }

    #[test]
    fn test_simple_entity_parsing() {
        let mut parser = TreeSitterVHDLParser::new().unwrap();
        let source = r#"
        entity counter is
            port(
                clk : in std_logic;
                reset : in std_logic;
                count : out std_logic_vector(7 downto 0)
            );
        end entity counter;
        "#;

        let tree = parser.parse(source);
        assert!(tree.is_ok());
        
        let tree = tree.unwrap();
        let root = tree.root_node();
        
        // Should have parsed successfully
        assert!(!root.has_error());
    }
}
