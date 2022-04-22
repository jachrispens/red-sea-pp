#![allow(dead_code)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::vec::Vec;

/// The preprocessor is conceptually built around streams of tokens.  
/// Streams flow from a Source, which produces atomic elements of 
/// information.  At some point the Source will 'dry up' and become
/// empty.  A Source may also define a contract around the atoms
/// it produces.  If that contract cannot be met, a Source is to 
/// return an Error.  Whether an Error is recoverable is defined
/// by the specific Source implementation.
trait Source<DatumType, ErrorType> {
    fn next(&mut self) -> Atom<DatumType, ErrorType>;
}

/// Atoms are the fundamental unit of information produces by sources.
/// However, there will be times when a valid atom cannot be produced,
/// either because there are no more atoms in the source (Empty) or 
/// because a valid atom cannot be produced (Error).
/// 
/// In essence, Atom is a combination of Result and Option.
enum Atom<DatumType, ErrorType> {
    Datum(DatumType),
    Error(ErrorType),
    Empty,
}

use self::Atom::*;

type PreprocessingTokenSource = dyn Source<PreprocessingToken, String>;
type PreprocessingAtom = Atom<PreprocessingToken, String>;

type Identifier = String;

#[derive(Clone, PartialEq, Eq)]
enum PreprocessingToken {
    HeaderName(HeaderKind, String),
    Identifier(Identifier),
    PreprocessingNumber(String),
    CharacterConstant(char),
    StringLiteral(String),
    Punctuator(Punctuator),
    OtherCharacter(char),
    /* Newline is not an actual token, but it is incluced here for two reasons:
        1. It is used in the grammar for directives.
        2. The replaced output will more closely resemble the input.
    */
    Newline,
}

/// Punctuator enumerates the C punctuator tokens.  The names (subjectively) capture
/// how the token is most often used in C.  Obviously a preprocessor
/// doesn't care about C semantics, but it help keep the names short and understandable to
/// those already familiar with C.  The order of the definitions matches the C17 draft
/// spec (N2176)
#[derive(Clone, PartialEq, Eq)]
enum Punctuator {
    /* [    */ ArrayIndexBegin,
    /* ]    */ ArrayIndexEnd,
    /* Whether a left paren is preceded by whitespace in a #define differentiates between
     * a function-like macro and an object macro whose replacement starts with a left paren */
    /* (    */
    LeftParen(Separation),
    /* )    */ RightParen,
    /* {    */ BlockBegin,
    /* }    */ BlockEnd,
    /* .    */ Member,
    /* ->   */ DerefMember,
    /* ++   */ Increment,
    /* --   */ Decrement,
    /* &    */ AddressOf,
    /* *    */ Deference,
    /* +    */ Add,
    /* -    */ Substract,
    /* ~    */ BitwiseNot,
    /* !    */ LogicalNot,
    /* /    */ Divide,
    /* %    */ Modulus,
    /* <<   */ ShiftLeft,
    /* >>   */ ShiftRight,
    /* <    */ LessThan,
    /* >    */ GreaterThan,
    /* <=   */ LessThanOrEquals,
    /* >=   */ GreaterThanOrEquals,
    /* ==   */ Equals,
    /* !=   */ NotEquals,
    /* ^    */ BitwiseXor,
    /* |    */ BitwiseOr,
    /* &&   */ LogicalAnd,
    /* ||   */ LogicalOr,
    /* ?    */ TernaryCondition,
    /* :    */ TernarySeparator,
    /* ;    */ StatementEnd,
    /* ...  */ VariadicParameters,
    /* =    */ Assignment,
    /* *=   */ MultiplyAndAssign,
    /* /=   */ DivideAndAssign,
    /* %=   */ ModulusAndAssign,
    /* +=   */ AddAndAssign,
    /* -=   */ SubstractAndAssign,
    /* <<=  */ ShiftLeftAndAssign,
    /* >>=  */ ShiftRightAndAssign,
    /* &=   */ BitwiseAndAndAssign,
    /* ^=   */ BitwiseXorAndAssign,
    /* |=   */ BitwiseOrAndAssign,
    /* ,    */ ParameterSeparator,
    /* #    */ PreprocessingDirective,
    /* ##   */ PreprocessingConcat,
    /* <:   */ ArrayIndexBeginDigraph,
    /* :>   */ ArrayIndexEndDigraph,
    /* <%   */ BlockBeginDigraph,
    /* %>   */ BlockEndDigraph,
    /* %:   */ PreprocessingDirectiveDigraph,
    /* %:%: */ PreprocessingConcatDigraph,
}

/// Separation indicates how token is separated
#[derive(Clone, PartialEq, Eq)]
enum Separation {
    Whitespace,
    None,
}

#[derive(Clone, PartialEq, Eq)]
enum HeaderKind {
    SystemPath,
    UserPath,
}

type Tokens = Vec<PreprocessingToken>;
type Parameters = Vec<Identifier>;
type ReplacedMacros = HashSet<Identifier>;

/// Macro replacement 
enum ScopedTokens {

}

enum Macro {
    Object(Tokens),
    Function(Parameters, Tokens),
}

struct Macros {
    definitions: HashMap<Identifier, Macro>,
}

impl Macros {
    fn new() -> Macros {
        Macros {
            definitions: HashMap::new(),
        }
    }
}

enum ExpandingToken {
    Token(PreprocessingToken),
    EndScope(Identifier),
}

struct MacroExpandingTokenSource<'stream> {
    macros: Macros,
    token_stream: &'stream mut PreprocessingTokenSource,
}

impl<'stream> MacroExpandingTokenSource<'stream> {
    fn new(
        macros: Macros,
        token_stream: &'stream mut PreprocessingTokenSource,
    ) -> MacroExpandingTokenSource {
        MacroExpandingTokenSource {
            macros,
            token_stream,
        }
    }
}

impl<'stream> Source<PreprocessingToken, String> for MacroExpandingTokenSource<'stream> {
    fn next(&mut self) -> PreprocessingAtom {
        match self.token_stream.next() {
            Datum(token) => Datum(token),
            Error(error) => Error(error),
            Empty => Empty,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PreprocessingToken::*;
    use super::*;
    use std::collections::VecDeque;

    type TokenQueue = VecDeque<PreprocessingToken>;

    struct TestTokenSource {
        token_queue: TokenQueue,
    }

    impl TestTokenSource {
        fn new(tokens: &Tokens) -> TestTokenSource {
            let mut token_queue = TokenQueue::new();
            for token in tokens {
                token_queue.push_back(token.clone());
            }
            TestTokenSource { token_queue }
        }
    }

    impl Source<PreprocessingToken, String> for TestTokenSource {
        fn next(&mut self) -> PreprocessingAtom {
            match self.token_queue.pop_front() {
                Some(token) => Datum(token),
                None => Empty
            }
        }
    }

    #[test]
    fn pass_through() {
        let test_tokens = vec![
            Identifier(String::from("A")),
            Identifier(String::from("B")),
            PreprocessingNumber(String::from("1234")),
        ];
        let mut test_stream = TestTokenSource::new(&test_tokens);
        let macros = Macros::new();
        let mut expanding_stream = MacroExpandingTokenSource::new(macros, &mut test_stream);
        let mut expansion_result = Vec::new();
        while let Datum(token) = expanding_stream.next() {
            expansion_result.push(token);
        }
        assert!(expansion_result == test_tokens);
    }
}
