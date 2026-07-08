use crate::diagnostic::Span;

#[derive(Debug, Clone, Default)]
pub struct Program {
    pub files: Vec<SourceFile>,
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: String,
    pub module: Option<String>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    App(App),
    Type(TypeDef),
    Store(Store),
    Task(Task),
    Test(Test),
}

#[derive(Debug, Clone)]
pub struct App {
    pub name: String,
    pub sections: Vec<Section>,
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: String,
    pub fields: Vec<Field>,
    pub sections: Vec<Section>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Store {
    pub name: String,
    pub ty: String,
    pub sections: Vec<Section>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub name: String,
    pub params: Vec<Param>,
    pub result: Option<String>,
    pub sections: Vec<Section>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Test {
    pub name: String,
    pub params: Vec<Param>,
    pub modifiers: Vec<String>,
    pub sections: Vec<Section>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamPermission {
    Borrow,
    Change,
    Consume,
}

impl ParamPermission {
    pub fn as_str(self) -> &'static str {
        match self {
            ParamPermission::Borrow => "borrow",
            ParamPermission::Change => "change",
            ParamPermission::Consume => "consume",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: String,
    pub permission: ParamPermission,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub lines: Vec<SectionLine>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SectionLine {
    pub text: String,
    pub span: Span,
}

impl Item {
    pub fn kind(&self) -> &'static str {
        match self {
            Item::App(_) => "app",
            Item::Type(_) => "type",
            Item::Store(_) => "store",
            Item::Task(_) => "task",
            Item::Test(_) => "test",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Item::App(item) => &item.name,
            Item::Type(item) => &item.name,
            Item::Store(item) => &item.name,
            Item::Task(item) => &item.name,
            Item::Test(item) => &item.name,
        }
    }

    pub fn span(&self) -> &Span {
        match self {
            Item::App(item) => &item.span,
            Item::Type(item) => &item.span,
            Item::Store(item) => &item.span,
            Item::Task(item) => &item.span,
            Item::Test(item) => &item.span,
        }
    }
}

impl Task {
    pub fn section(&self, name: &str) -> Option<&Section> {
        find_section(&self.sections, name)
    }
}

impl Test {
    pub fn section(&self, name: &str) -> Option<&Section> {
        find_section(&self.sections, name)
    }
}

impl App {
    pub fn section(&self, name: &str) -> Option<&Section> {
        find_section(&self.sections, name)
    }
}

impl TypeDef {
    pub fn section(&self, name: &str) -> Option<&Section> {
        find_section(&self.sections, name)
    }
}

impl Store {
    pub fn section(&self, name: &str) -> Option<&Section> {
        find_section(&self.sections, name)
    }
}

pub fn find_section<'a>(sections: &'a [Section], name: &str) -> Option<&'a Section> {
    sections.iter().find(|section| section.name == name)
}
