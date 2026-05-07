use serde::Serialize;
use xmlparser::{ElementEnd, EntityDefinition, ExternalId, StrSpan, Token};

/// Wrapper around xmlparser::StrSpan to implement Serialize
#[derive(Serialize)]
pub struct SerStrSpan<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
}

impl<'a> From<StrSpan<'a>> for SerStrSpan<'a> {
    fn from(s: StrSpan<'a>) -> Self {
        Self {
            text: s.as_str(),
            start: s.start(),
            end: s.end(),
        }
    }
}

/// Wrapper around xmlparser::ExternalId to implement Serialize
#[derive(Serialize)]
pub enum SerExternalId<'a> {
    System(SerStrSpan<'a>),
    Public(SerStrSpan<'a>, SerStrSpan<'a>),
}

impl<'a> From<ExternalId<'a>> for SerExternalId<'a> {
    fn from(id: ExternalId<'a>) -> Self {
        match id {
            ExternalId::System(s) => Self::System(s.into()),
            ExternalId::Public(a, b) => Self::Public(a.into(), b.into()),
        }
    }
}

/// Wrapper around xmlparser::EntityDefinition to implement Serialize
#[derive(Serialize)]
pub enum SerEntityDefinition<'a> {
    EntityValue(SerStrSpan<'a>),
    ExternalId(SerExternalId<'a>),
}

impl<'a> From<EntityDefinition<'a>> for SerEntityDefinition<'a> {
    fn from(d: EntityDefinition<'a>) -> Self {
        match d {
            EntityDefinition::EntityValue(s) => Self::EntityValue(s.into()),
            EntityDefinition::ExternalId(id) => Self::ExternalId(id.into()),
        }
    }
}

/// Wrapper around xmlparser::ElementEnd to implement Serialize
#[derive(Serialize)]
pub enum SerElementEnd<'a> {
    Open,
    Close {
        prefix: SerStrSpan<'a>,
        local: SerStrSpan<'a>,
    },
    Empty,
}

impl<'a> From<ElementEnd<'a>> for SerElementEnd<'a> {
    fn from(e: ElementEnd<'a>) -> Self {
        match e {
            ElementEnd::Open => Self::Open,
            ElementEnd::Close(prefix, local) => Self::Close {
                prefix: prefix.into(),
                local: local.into(),
            },
            ElementEnd::Empty => Self::Empty,
        }
    }
}

/// Wrapper around xmlparser::Token to implement Serialize
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum SerToken<'a> {
    Declaration {
        version: SerStrSpan<'a>,
        encoding: Option<SerStrSpan<'a>>,
        standalone: Option<bool>,
        span: SerStrSpan<'a>,
    },
    ProcessingInstruction {
        target: SerStrSpan<'a>,
        content: Option<SerStrSpan<'a>>,
        span: SerStrSpan<'a>,
    },
    Comment {
        text: SerStrSpan<'a>,
        span: SerStrSpan<'a>,
    },
    DtdStart {
        name: SerStrSpan<'a>,
        external_id: Option<SerExternalId<'a>>,
        span: SerStrSpan<'a>,
    },
    EmptyDtd {
        name: SerStrSpan<'a>,
        external_id: Option<SerExternalId<'a>>,
        span: SerStrSpan<'a>,
    },
    EntityDeclaration {
        name: SerStrSpan<'a>,
        definition: SerEntityDefinition<'a>,
        span: SerStrSpan<'a>,
    },
    DtdEnd {
        span: SerStrSpan<'a>,
    },
    ElementStart {
        prefix: SerStrSpan<'a>,
        local: SerStrSpan<'a>,
        span: SerStrSpan<'a>,
    },
    Attribute {
        prefix: SerStrSpan<'a>,
        local: SerStrSpan<'a>,
        value: SerStrSpan<'a>,
        span: SerStrSpan<'a>,
    },
    ElementEnd {
        end: SerElementEnd<'a>,
        span: SerStrSpan<'a>,
    },
    Text {
        text: SerStrSpan<'a>,
    },
    Cdata {
        text: SerStrSpan<'a>,
        span: SerStrSpan<'a>,
    },
}

impl<'a> From<Token<'a>> for SerToken<'a> {
    fn from(t: Token<'a>) -> Self {
        match t {
            Token::Declaration {
                version,
                encoding,
                standalone,
                span,
            } => Self::Declaration {
                version: version.into(),
                encoding: encoding.map(Into::into),
                standalone,
                span: span.into(),
            },
            Token::ProcessingInstruction {
                target,
                content,
                span,
            } => Self::ProcessingInstruction {
                target: target.into(),
                content: content.map(Into::into),
                span: span.into(),
            },
            Token::Comment { text, span } => Self::Comment {
                text: text.into(),
                span: span.into(),
            },
            Token::DtdStart {
                name,
                external_id,
                span,
            } => Self::DtdStart {
                name: name.into(),
                external_id: external_id.map(Into::into),
                span: span.into(),
            },
            Token::EmptyDtd {
                name,
                external_id,
                span,
            } => Self::EmptyDtd {
                name: name.into(),
                external_id: external_id.map(Into::into),
                span: span.into(),
            },
            Token::EntityDeclaration {
                name,
                definition,
                span,
            } => Self::EntityDeclaration {
                name: name.into(),
                definition: definition.into(),
                span: span.into(),
            },
            Token::DtdEnd { span } => Self::DtdEnd { span: span.into() },
            Token::ElementStart {
                prefix,
                local,
                span,
            } => Self::ElementStart {
                prefix: prefix.into(),
                local: local.into(),
                span: span.into(),
            },
            Token::Attribute {
                prefix,
                local,
                value,
                span,
            } => Self::Attribute {
                prefix: prefix.into(),
                local: local.into(),
                value: value.into(),
                span: span.into(),
            },
            Token::ElementEnd { end, span } => Self::ElementEnd {
                end: end.into(),
                span: span.into(),
            },
            Token::Text { text } => Self::Text { text: text.into() },
            Token::Cdata { text, span } => Self::Cdata {
                text: text.into(),
                span: span.into(),
            },
        }
    }
}
