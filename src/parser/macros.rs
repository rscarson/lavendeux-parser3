/// Skip all EOL tokens
macro_rules! skip_eol {
    ($tokens:expr) => {
        terminal!(EOL*, $tokens)
    };
}

/// Attempt to match a single token
macro_rules! terminal {
    (& $type:ident $(| $($subtype:ident)|+)?, $tokens:expr $(, $skip_eol:expr)?) => {{
        $(
            $skip_eol;
        )?
        $tokens.try_peek_a(&[$crate::lexer::Rule::$type $(, $($crate::lexer::Rule::$subtype,)+)?]).cloned()
    }};
    ($type:ident $(| $($subtype:ident)|+)? ?, $tokens:expr $(, $skip_eol:expr)?) => {{
        $(
            $skip_eol;
        )?
        match terminal!(&$type $(| $($subtype)|+)?, $tokens) {
            Some(_) => $tokens.pop(),
            None => None
        }
    }};
    ($type:ident $(| $($subtype:ident)|+)? *, $tokens:expr $(, $skip_eol:expr)?) => {{
        $(
            $skip_eol;
        )?
        let mut v = Vec::new();
        while let Some(t) = terminal!($type$(| $($subtype)|+)? ?, $tokens) {
            v.push(t)
        }
        v
    }};
    ($type:ident $(| $($subtype:ident)|+)? +, $tokens:expr $(, $skip_eol:expr)?) => {{
        $(
            $skip_eol;
        )?
        match terminal!($type $(| $($subtype)|+)?, $tokens) {
            Some(t) => {
                let mut v = vec![t];
                while let Some(t) = terminal!($type $(| $($subtype)|+)? ?, $tokens) {
                    v.push(t)
                }
                Some(v)
            }
            None => None
        }
    }};
    ($type:ident $(| $($subtype:ident)|+)?, $tokens:expr $(, $skip_eol:expr)?) => {{
        $(
            $skip_eol;
        )?
        $tokens.try_pop_a(&[$crate::lexer::Rule::$type $(, $($crate::lexer::Rule::$subtype,)+)?])
    }};
}

macro_rules! build_nt {
    ($type:ident, $tokens:expr) => {{
        match $tokens.len() == 0 {
            true => None,
            _ => {
                #[cfg(feature = "debug_compiler_internal_parser")]
                {
                    println!(
                        "{}Parsing {}: next={:?}",
                        " ".repeat($tokens.depth()),
                        stringify!($type),
                        $tokens.peek()
                    );
                    match $type::parse($tokens) {
                        Some(t) => {
                            println!(
                                "{}Success ({:?})!",
                                " ".repeat($tokens.depth() + 1),
                                stringify!($type)
                            );
                            Some(t)
                        }
                        None => None,
                    }
                }
                #[cfg(not(feature = "debug_compiler_internal_parser"))]
                $type::parse($tokens)
            }
        }
    }};
}

/// Attempt to match a NT
macro_rules! non_terminal {
    (! $type:ident, $tokens:expr $(, $skip_eol:expr)?) => {{
        $(
            $skip_eol;
        )?
        match non_terminal!($type?, $tokens) {
            None => Ok(()),
            Some(nt) => Err(Error::Syntax {
                expected: vec![],
                unexpected: vec![nt.token()]
            })
        }
    }};
    ($type:ident ?, $tokens:expr $(, $skip_eol:expr)?) => {{
        $(
            $skip_eol;
        )?
        build_nt!($type, $tokens)
    }};
    ($type:ident *, $tokens:expr $(, $skip_eol:expr)?) => {{
        $(
            $skip_eol;
        )?
        let mut v = Vec::new();
        while let Some(t) = non_terminal!($type?, $tokens) {
            v.push(t)
        }
        v
    }};
    ($type:ident +, $tokens:expr $(, $skip_eol:expr)?) => {{
        $(
            $skip_eol;
        )?
        match non_terminal!($type, $tokens) {
            Ok(t) => {
                let mut v = vec![t];
                while let Some(t) = non_terminal!($type?, $tokens) {
                    v.push(t)
                }
                Ok(v)
            }
            Err(e) =>  Err(e)
        }
    }};
    ($type:ident  $(| $($subtype:ident)|+)?, $tokens:expr $(, $skip_eol:expr)?) => {{
        $(
            $skip_eol;
        )?
        match build_nt!($type, $tokens) {
            Some(t) => Some(t),
            None => {
                #[allow(unused_mut)]
                let mut result = None;
                $(
                    'block: { $(
                        match build_nt!($subtype, $tokens) {
                            Some(t) => {
                                result = Some(t);
                                break 'block;
                            },
                            _ => {}
                        }
                    )+ }
                )?

                if result.is_none() {
                    $tokens.revert_transaction();
                }
                result
            }
        }
    }};
}

macro_rules! error_node {
    ($error:expr) => {
        Some(Node::Error($error))
    };
}

macro_rules! define_node {
    (
        $name:ident ( $($an:ident : $at:ty),* $(,)?) {
            $($docstr:literal)*
            build($bstack_arg:ident) $bblock:block
            compile($cselfarg:ident, $ccompilerarg:ident) $cblock:block
            into_node($nselfarg:ident) $nblock:block
            into_owned($oselfarg:ident) $oblock:block
        }
    ) => {
        $(#[doc = $docstr])*
        #[derive(Clone, Debug)]
        pub struct $name<'source> {
            $(pub $an: $at,)*
            pub token: $crate::lexer::Token<'source>,
        }
        impl<'source> IntoOwned for $name<'source> {
            type Owned = $name<'static>;
            fn into_owned(self) -> Self::Owned {
                let $oselfarg = self;
                $oblock
            }
        }
        impl<'source> $crate::parser::ParserNode<'source> for $name<'source> {
            fn into_node(self) -> $crate::parser::Node<'source> {
                let $nselfarg = self;
                $nblock
            }
            fn parse($bstack_arg: &mut $crate::lexer::Stack<'source>) -> Option<$crate::parser::Node<'source>> $bblock

            fn compile(self, $ccompilerarg: &mut $crate::compiler::Compiler) -> Result<(), $crate::compiler::CompilerError> {
                let $cselfarg = self;
                $cblock
            }
        }
    };
}

macro_rules! pratt_node {
    (
        $name:ident ( $($an:ident : $at:ty),* $(,)?) {
            $($docstr:literal)*
            build($bt_arg:ident, $bl_arg:ident, $bo_arg:ident $(, $br_arg:ident)?) $bblock:block
            compile($cselfarg:ident, $ccompilerarg:ident) $cblock:block
            into_node($nselfarg:ident) $nblock:block
            into_owned($oselfarg:ident) $oblock:block
        }
    ) => {
        $(#[doc = $docstr])*
        #[derive(Clone, Debug)]
        pub struct $name<'source> {
            $(pub $an: $at,)*
            pub token: $crate::lexer::Token<'source>,
        }
        impl<'source> IntoOwned for $name<'source> {
            type Owned = $name<'static>;
            fn into_owned(self) -> Self::Owned {
                let $oselfarg = self;
                $oblock
            }
        }
        impl<'source> $name<'source> {
            pub fn into_node(self) -> $crate::parser::Node<'source> {
                let $nselfarg = self;
                $nblock
            }
            pub fn parse(
                mut $bt_arg: $crate::lexer::Token<'source>,
                $bl_arg: Node<'source>,
                $bo_arg: Node<'source>
                $(, $br_arg: Node<'source>)?
            ) -> Option<$crate::parser::Node<'source>> $bblock

            pub fn compile(self, $ccompilerarg: &mut $crate::compiler::Compiler) -> Result<(), $crate::compiler::CompilerError> {
                let $cselfarg = self;
                $cblock
            }
        }
    };
}

macro_rules! pratt_node_silent {
    (
        $name:ident {
            $($docstr:literal)*
            build($bt_arg:ident, $bl_arg:ident, $bo_arg:ident $(, $br_arg:ident)?) $bblock:block
        }
    ) => {
        $(#[doc = $docstr])*
        #[derive(Clone, Debug)]
        pub struct $name { }
        impl $name {
            pub fn parse<'source>(mut $bt_arg: $crate::lexer::Token<'source>, $bl_arg: Node<'source>, $bo_arg: Node<'source>$(, $br_arg: Node<'source>)?) -> Option<$crate::parser::Node<'source>> $bblock
        }
    };
}

macro_rules! node_silent {
    (
        $name:ident {
            $($docstr:literal)*
            build($bstack_arg:ident) $bblock:block
        }
    ) => {
        $(#[doc = $docstr])*
        #[derive(Clone, Debug)]
        pub struct $name<'source>{
            spoopy: std::marker::PhantomData<&'source ()>
        }
        impl<'source> $name<'source> {
            pub fn parse($bstack_arg: &mut $crate::lexer::Stack<'source>) -> Option<$crate::parser::Node<'source>> $bblock
        }
    };
}

macro_rules! define_parser {
    ($($name:ident : $src:ident),+ $(,)?) => {
        #[derive(Clone)]
        pub enum Node<'source> {
            Error($crate::parser::ParserError),
            $(
                $name(Box<$src<'source>>),
            )+
        }

        impl<'source> Node<'source> {
            pub fn token(&self) -> &$crate::lexer::Token<'source> {
                match self {
                    Self::Error(e) => e.token(),
                    $(
                        Self::$name(n) => &n.token,
                    )+
                }
            }

            pub fn compile(self, compiler: &mut $crate::compiler::Compiler) -> Result<(), $crate::compiler::CompilerError> {
                match self {
                    Self::Error(e) => Err(e.into()),
                    $(
                        Self::$name(n) => n.compile(compiler),
                    )+
                }
            }
        }

        impl<'source> $crate::traits::IntoOwned for Node<'source> {
            type Owned = Node<'static>;
            fn into_owned(self) -> Self::Owned {
                match self {
                    Self::Error(e) => Self::Owned::Error(e),
                    $(
                        Self::$name(n) => Self::Owned::$name(Box::new(n.into_owned())),
                    )+
                }
            }
        }

        impl std::fmt::Debug for Node<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::Error(e) => write!(f, "{:?}", e),
                    $(
                        Self::$name(n) => write!(f, "{:?}", n),
                    )+
                }
            }
        }
    };
}
