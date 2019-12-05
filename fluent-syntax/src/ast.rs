#[derive(Debug, PartialEq)]
pub struct Resource {
    pub body: Box<[ResourceEntry]>,
}

#[derive(Debug, PartialEq)]
pub enum ResourceEntry {
    Entry(Entry),
}

#[derive(Debug, PartialEq)]
pub enum Entry {
    Message(Message),
    Term(Term),
    Comment(Comment),
}

#[derive(Debug, PartialEq)]
pub struct Message {
    pub id: Identifier,
    pub value: Option<Pattern>,
    pub attributes: Box<[Attribute]>,
    pub comment: Option<Comment>,
}

#[derive(Debug, PartialEq)]
pub struct Term {
    pub id: Identifier,
    pub value: Pattern,
    pub attributes: Box<[Attribute]>,
    pub comment: Option<Comment>,
}

#[derive(Debug, PartialEq)]
pub struct Pattern {
    pub elements: Box<[PatternElement]>,
}

#[derive(Debug, PartialEq)]
pub enum PatternElement {
    TextElement(String),
}

#[derive(Debug, PartialEq)]
pub struct Attribute {
    pub id: Identifier,
    pub value: Pattern,
}

#[derive(Debug, PartialEq)]
pub struct Identifier {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub enum CommentType {
    Regular,
    Group,
    Resource,
}

#[derive(Debug, PartialEq)]
pub struct Comment {
    pub comment_type: CommentType,
    pub content: Box<[String]>,
}
