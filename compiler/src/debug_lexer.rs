use crate::lexer::{Lexer, TokenKind};
use crate::error::Result;

fn main() -> Result<()> {
    let input = r#"
let numbers = [1, 2, 3]

for num in numbers {
}
"#;
    
    let mut lexer = Lexer::new(input);
    loop {
        let token = lexer.next_token()?;
        println!("{:?}", token);
        if token.kind == TokenKind::Eof {
            break;
        }
    }
    Ok(())
}