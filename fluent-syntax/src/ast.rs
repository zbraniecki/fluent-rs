#[derive(Debug, PartialEq, Clone)]
pub struct Resource<S> {
    pub body: Vec<ResourceEntry<S>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ResourceEntry<S> {
    Entry(Entry<S>),
    Junk(S),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Entry<S> {
    Message(Message<S>),
    Term(Term<S>),
    Comment(Comment<S>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Message<S> {
    pub id: Identifier<S>,
    pub value: Option<Pattern<S>>,
    pub attributes: Vec<Attribute<S>>,
    pub comment: Option<Comment<S>>,
}

impl Message<String> {
    pub fn borrowed<'s>(&'s self) -> Message<&'s str> {
        Message {
            id: self.id.borrowed(),
            value: self.value.as_ref().map(|v| v.borrowed()),
            attributes: vec![],
            comment: None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Term<S> {
    pub id: Identifier<S>,
    pub value: Pattern<S>,
    pub attributes: Vec<Attribute<S>>,
    pub comment: Option<Comment<S>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pattern<S> {
    pub elements: Vec<PatternElement<S>>,
}

impl Pattern<String> {
    pub fn borrowed<'s>(&'s self) -> Pattern<&'s str> {
        let elements = self.elements.iter().map(|elem| elem.borrowed()).collect();
        Pattern { elements }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PatternElement<S> {
    TextElement(S),
    Placeable(Expression<S>),
}

impl PatternElement<String> {
    pub fn borrowed<'s>(&'s self) -> PatternElement<&'s str> {
        match self {
            Self::TextElement(ref s) => PatternElement::TextElement(s.as_str()),
            Self::Placeable(_) => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Attribute<S> {
    pub id: Identifier<S>,
    pub value: Pattern<S>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier<S> {
    pub name: S,
}

impl Identifier<String> {
    pub fn borrowed<'s>(&'s self) -> Identifier<&'s str> {
        Identifier { name: &self.name }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Variant<S> {
    pub key: VariantKey<S>,
    pub value: Pattern<S>,
    pub default: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum VariantKey<S> {
    Identifier { name: S },
    NumberLiteral { value: S },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Comment<S> {
    Comment { content: Vec<S> },
    GroupComment { content: Vec<S> },
    ResourceComment { content: Vec<S> },
}

#[derive(Debug, PartialEq, Clone)]
pub enum InlineExpression<S> {
    StringLiteral {
        value: S,
    },
    NumberLiteral {
        value: S,
    },
    FunctionReference {
        id: Identifier<S>,
        arguments: Option<CallArguments<S>>,
    },
    MessageReference {
        id: Identifier<S>,
        attribute: Option<Identifier<S>>,
    },
    TermReference {
        id: Identifier<S>,
        attribute: Option<Identifier<S>>,
        arguments: Option<CallArguments<S>>,
    },
    VariableReference {
        id: Identifier<S>,
    },
    Placeable {
        expression: Box<Expression<S>>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct CallArguments<S> {
    pub positional: Vec<InlineExpression<S>>,
    pub named: Vec<NamedArgument<S>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NamedArgument<S> {
    pub name: Identifier<S>,
    pub value: InlineExpression<S>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression<S> {
    InlineExpression(InlineExpression<S>),
    SelectExpression {
        selector: InlineExpression<S>,
        variants: Vec<Variant<S>>,
    },
}
