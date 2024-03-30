use super::Rule;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Operator(Vec<Rule>),
    Symbol(Vec<Rule>),
    Keyword(Vec<Rule>),
    Identifier,
    Literal,
    EOL,
    EOI,
}
impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Operator(r) => {
                let inner = r
                    .iter()
                    .map(|r| format!("`{r}`"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "Operator({inner})")
            }
            Category::Symbol(r) => {
                let inner = r
                    .iter()
                    .map(|r| format!("{r}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "Symbol(`{inner}`)")
            }
            Category::Keyword(r) => {
                let inner = r
                    .iter()
                    .map(|r| format!("`{r}`"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "Keyword({inner})")
            }
            Category::Identifier => write!(f, "identifier"),
            Category::Literal => write!(f, "literal value"),
            Category::EOL => write!(f, "linebreak"),
            Category::EOI => write!(f, "end of input"),
        }
    }
}
impl Category {
    pub fn from_rule(rule: Rule) -> Option<Self> {
        Some(match rule {
            Rule::EOI => Category::EOI,
            Rule::EOL => Category::EOL,

            Rule::LParen
            | Rule::RParen
            | Rule::LBrace
            | Rule::RBrace
            | Rule::LBrack
            | Rule::RBrack
            | Rule::Comma
            | Rule::Colon
            | Rule::Range
            | Rule::Dot
            | Rule::Question
            | Rule::Decorator => Category::Symbol(vec![rule]),

            Rule::Assign
            | Rule::AssignAdd
            | Rule::AssignSub
            | Rule::AssignPow
            | Rule::AssignMul
            | Rule::AssignDiv
            | Rule::AssignMod
            | Rule::AssignOr
            | Rule::AssignAnd
            | Rule::AssignXor
            | Rule::AssignSL
            | Rule::AssignSR
            | Rule::Inc
            | Rule::Dec
            | Rule::Add
            | Rule::Sub
            | Rule::Pow
            | Rule::Mul
            | Rule::Div
            | Rule::Mod
            | Rule::BitwiseNot
            | Rule::BitwiseOr
            | Rule::BitwiseAnd
            | Rule::Xor
            | Rule::SL
            | Rule::SR
            | Rule::LogicalOr
            | Rule::LogicalAnd
            | Rule::LogicalNot
            | Rule::SEq
            | Rule::SNe
            | Rule::Eq
            | Rule::Ne
            | Rule::Le
            | Rule::Ge
            | Rule::Lt
            | Rule::Gt
            | Rule::FatArrow => Category::Operator(vec![rule]),

            Rule::If
            | Rule::Then
            | Rule::Else
            | Rule::For
            | Rule::In
            | Rule::Do
            | Rule::Where
            | Rule::Switch
            | Rule::Return
            | Rule::Continue
            | Rule::Break
            | Rule::Delete
            | Rule::As
            | Rule::Contains
            | Rule::Matches
            | Rule::Is
            | Rule::StartsWith
            | Rule::EndsWith => Category::Keyword(vec![rule]),

            Rule::LiteralIdent => Category::Identifier,

            Rule::LiteralConstPi
            | Rule::LiteralConstE
            | Rule::LiteralConstTau
            | Rule::LiteralConstNil
            | Rule::LiteralConstTrue
            | Rule::LiteralConstFalse
            | Rule::LiteralInt
            | Rule::LiteralRadix
            | Rule::LiteralPrefixedCurrency
            | Rule::LiteralSuffixedCurrency
            | Rule::LiteralFloat
            | Rule::LiteralRegex
            | Rule::LiteralString => Category::Literal,

            _ => return None,
        })
    }

    pub fn from_ruleset(rules: &[Rule]) -> Vec<Self> {
        let categories = rules
            .iter()
            .filter_map(|r| Category::from_rule(*r))
            .collect::<Vec<_>>();

        let mut symbols = std::collections::HashSet::new();
        let mut keywords = std::collections::HashSet::new();
        let mut operators = std::collections::HashSet::new();
        let mut set = std::collections::HashSet::new();

        for c in categories.into_iter() {
            match c {
                Category::Symbol(r) => symbols.extend(r),
                Category::Keyword(r) => keywords.extend(r),
                Category::Operator(r) => operators.extend(r),
                _ => {
                    set.insert(c);
                }
            }
        }

        let mut categories = vec![];
        if !symbols.is_empty() {
            categories.push(Category::Symbol(symbols.into_iter().collect()));
        }
        if !keywords.is_empty() {
            categories.push(Category::Keyword(keywords.into_iter().collect()));
        }
        if !operators.is_empty() {
            categories.push(Category::Operator(operators.into_iter().collect()));
        }
        categories.extend(set.into_iter());
        categories
    }

    pub fn many_to_string(this: &Vec<Self>) -> String {
        format!(
            "{}",
            this.iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    pub fn format_rules(this: &Vec<Rule>) -> String {
        let categories = Category::from_ruleset(this);
        Category::many_to_string(&categories)
    }
}
