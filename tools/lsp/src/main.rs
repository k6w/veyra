use anyhow::Result;
use dashmap::DashMap;
use ropey::Rope;

use std::sync::Arc;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

// Import from the main compiler
use veyra_compiler::{
    ast::*,
    error::VeyraError,
    lexer::{Lexer, Token, TokenKind},
    parser::Parser as VeyraParser,
};

#[derive(Debug)]
struct DocumentInfo {
    rope: Rope,
    version: i32,
    diagnostics: Vec<Diagnostic>,
    symbols: Vec<DocumentSymbol>,
    tokens: Vec<Token>,
    ast: Option<Program>,
}

impl DocumentInfo {
    fn new(text: String, version: i32) -> Self {
        let rope = Rope::from_str(&text);
        let mut info = Self {
            rope,
            version,
            diagnostics: Vec::new(),
            symbols: Vec::new(),
            tokens: Vec::new(),
            ast: None,
        };
        info.analyze();
        info
    }

    fn update(&mut self, changes: Vec<TextDocumentContentChangeEvent>, version: i32) {
        self.version = version;

        for change in changes {
            if let Some(range) = change.range {
                let start_idx = self.rope.line_to_char(range.start.line as usize)
                    + range.start.character as usize;
                let end_idx =
                    self.rope.line_to_char(range.end.line as usize) + range.end.character as usize;
                self.rope.remove(start_idx..end_idx);
                self.rope.insert(start_idx, &change.text);
            } else {
                // Full document update
                self.rope = Rope::from_str(&change.text);
            }
        }

        self.analyze();
    }

    fn analyze(&mut self) {
        let text = self.rope.to_string();
        self.diagnostics.clear();
        self.symbols.clear();
        self.tokens.clear();
        self.ast = None;

        // Tokenize
        let mut lexer = Lexer::new(&text);
        match lexer.tokenize() {
            Ok(tokens) => {
                self.tokens = tokens.clone();

                // Parse
                let mut parser = VeyraParser::new(tokens);
                match parser.parse() {
                    Ok(ast) => {
                        self.ast = Some(ast.clone());
                        self.extract_symbols(&ast);
                    }
                    Err(e) => {
                        self.add_diagnostic_from_error(&e);
                    }
                }
            }
            Err(e) => {
                self.add_diagnostic_from_error(&e);
            }
        }
    }

    fn add_diagnostic_from_error(&mut self, error: &VeyraError) {
        // Convert VeyraError to LSP Diagnostic
        let diagnostic = Diagnostic {
            range: Range {
                start: Position {
                    line: 0, // TODO: Extract line info from error
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("veyra".to_string()),
            message: error.to_string(),
            related_information: None,
            tags: None,
            data: None,
        };

        self.diagnostics.push(diagnostic);
    }

    fn extract_symbols(&mut self, program: &Program) {
        for item in &program.items {
            self.extract_symbol_from_item(item);
        }
    }

    fn extract_symbol_from_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => {
                let symbol = DocumentSymbol {
                    name: func.name.clone(),
                    detail: Some(format!("function({} parameters)", func.parameters.len())),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    #[allow(deprecated)]
                    deprecated: None,
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        }, // TODO: Track positions
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                    selection_range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                    children: None,
                };
                self.symbols.push(symbol);
            }
            Item::Statement(Statement::VariableDeclaration(var_decl)) => {
                let symbol = DocumentSymbol {
                    name: var_decl.name.clone(),
                    detail: Some("variable".to_string()),
                    kind: SymbolKind::VARIABLE,
                    tags: None,
                    #[allow(deprecated)]
                    deprecated: None,
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                    selection_range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                    children: None,
                };
                self.symbols.push(symbol);
            }
            _ => {}
        }
    }

    fn get_text_at_position(&self, position: Position) -> Option<String> {
        let line_idx = position.line as usize;
        let char_idx = position.character as usize;

        if line_idx >= self.rope.len_lines() {
            return None;
        }

        let line = self.rope.line(line_idx);
        let line_str = line.to_string();

        // Find word boundaries around the position
        let chars: Vec<char> = line_str.chars().collect();

        if char_idx >= chars.len() {
            return None;
        }

        let mut start = char_idx;
        let mut end = char_idx;

        // Find start of word
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        // Find end of word
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }
}

struct VeyraLanguageServer {
    client: Client,
    documents: Arc<DashMap<Url, DocumentInfo>>,
}

impl VeyraLanguageServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DashMap::new()),
        }
    }

    async fn publish_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    fn get_completions_for_context(&self, _uri: &Url, _position: Position) -> Vec<CompletionItem> {
        // Built-in keywords
        let mut completions = vec![
            CompletionItem {
                label: "let".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Variable declaration".to_string()),
                documentation: Some(Documentation::String("Declare a new variable".to_string())),
                insert_text: Some("let ${1:name} = ${2:value}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "fn".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Function declaration".to_string()),
                documentation: Some(Documentation::String("Declare a new function".to_string())),
                insert_text: Some("fn ${1:name}(${2:params}) {\n\t${3:body}\n}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "if".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Conditional statement".to_string()),
                insert_text: Some("if ${1:condition} {\n\t${2:body}\n}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "while".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("While loop".to_string()),
                insert_text: Some("while ${1:condition} {\n\t${2:body}\n}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "for".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("For loop".to_string()),
                insert_text: Some("for ${1:item} in ${2:iterable} {\n\t${3:body}\n}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ];

        // Built-in functions
        let builtin_functions = vec![
            ("print", "Print a value to stdout"),
            ("len", "Get the length of an array or string"),
            ("str", "Convert a value to string"),
            ("push", "Add an element to an array"),
            ("pop", "Remove and return the last element of an array"),
        ];

        for (name, description) in builtin_functions {
            completions.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Built-in function".to_string()),
                documentation: Some(Documentation::String(description.to_string())),
                insert_text: Some(format!("{}(${{1}})", name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }

        // TODO: Add context-specific completions (variables, functions from current document)

        completions
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for VeyraLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Veyra Language Server".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), " ".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: None,
                    work_done_progress_options: Default::default(),
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_highlight_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_range_formatting_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            work_done_progress_options: Default::default(),
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::COMMENT,
                                ],
                                token_modifiers: vec![],
                            },
                            range: Some(true),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                        },
                    ),
                ),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Veyra Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        let text = params.text_document.text;

        let document_info = DocumentInfo::new(text, version);
        let diagnostics = document_info.diagnostics.clone();

        self.documents.insert(uri.clone(), document_info);
        self.publish_diagnostics(uri, diagnostics).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        if let Some(mut document) = self.documents.get_mut(&uri) {
            document.update(params.content_changes, version);
            let diagnostics = document.diagnostics.clone();
            drop(document); // Release the lock

            self.publish_diagnostics(uri, diagnostics).await;
        }
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "Document saved!")
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let completions = self.get_completions_for_context(&uri, position);

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(document) = self.documents.get(&uri) {
            if let Some(word) = document.get_text_at_position(position) {
                let hover_content = match word.as_str() {
                    "print" => "Built-in function: print(value) - Print a value to stdout",
                    "len" => {
                        "Built-in function: len(collection) - Get the length of an array or string"
                    }
                    "str" => "Built-in function: str(value) - Convert a value to string",
                    "let" => "Keyword: let - Declare a new variable",
                    "fn" => "Keyword: fn - Declare a new function",
                    "if" => "Keyword: if - Conditional statement",
                    "while" => "Keyword: while - While loop",
                    "for" => "Keyword: for - For loop",
                    _ => return Ok(None),
                };

                return Ok(Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(
                        hover_content.to_string(),
                    )),
                    range: None,
                }));
            }
        }

        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> LspResult<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        if let Some(document) = self.documents.get(&uri) {
            let symbols = document.symbols.clone();
            return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
        }

        Ok(None)
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> LspResult<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;

        if let Some(_document) = self.documents.get(&uri) {
            // TODO: Implement actual formatting using the formatter tool
            // For now, return empty (no changes)
            return Ok(Some(vec![]));
        }

        Ok(None)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> LspResult<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;

        if let Some(document) = self.documents.get(&uri) {
            let mut tokens_data: Vec<SemanticToken> = Vec::new();
            let mut prev_line = 0;
            let mut prev_char = 0;

            // Convert our tokens to semantic tokens
            for token in &document.tokens {
                let token_type = match token.kind {
                    TokenKind::Fn
                    | TokenKind::Let
                    | TokenKind::If
                    | TokenKind::While
                    | TokenKind::For => 0, // KEYWORD
                    TokenKind::String(_) => 1, // STRING
                    TokenKind::Integer(_) | TokenKind::Float(_) => 2, // NUMBER
                    TokenKind::Identifier => {
                        // Determine if it's a function or variable
                        // TODO: More sophisticated analysis
                        4 // VARIABLE for now
                    }
                    _ => continue, // Skip other tokens
                };

                let line = token.line as u32;
                let character = token.column as u32;
                let length = match &token.kind {
                    TokenKind::String(s) => s.len() + 2, // +2 for quotes
                    TokenKind::Integer(_) => token.lexeme.len(),
                    TokenKind::Float(_) => token.lexeme.len(),
                    TokenKind::Identifier => token.lexeme.len(),
                    _ => token.lexeme.len(),
                };

                tokens_data.push(SemanticToken {
                    delta_line: line - prev_line,
                    delta_start: if line == prev_line {
                        character - prev_char
                    } else {
                        character
                    },
                    length: length as u32,
                    token_type,
                    token_modifiers_bitset: 0,
                });

                prev_line = line;
                prev_char = character;
            }

            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: tokens_data,
            })));
        }

        Ok(None)
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> LspResult<Option<SemanticTokensRangeResult>> {
        // For now, just return the full semantic tokens for the range
        // TODO: Optimize to only return tokens within the requested range
        let full_params = SemanticTokensParams {
            text_document: params.text_document,
            partial_result_params: params.partial_result_params,
            work_done_progress_params: params.work_done_progress_params,
        };

        match self.semantic_tokens_full(full_params).await {
            Ok(Some(SemanticTokensResult::Tokens(tokens))) => {
                Ok(Some(SemanticTokensRangeResult::Tokens(tokens)))
            }
            Ok(Some(SemanticTokensResult::Partial(_))) => {
                // Handle partial results if needed
                Ok(None)
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> LspResult<Option<Vec<DocumentHighlight>>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(document) = self.documents.get(&uri) {
            if let Some(word) = document.get_text_at_position(position) {
                let mut highlights = Vec::new();

                // Find all occurrences of the word in the document
                let text = document.rope.to_string();
                let lines: Vec<&str> = text.lines().collect();

                for (line_idx, line) in lines.iter().enumerate() {
                    let mut start = 0;
                    while let Some(pos) = line[start..].find(&word) {
                        let char_start = start + pos;
                        highlights.push(DocumentHighlight {
                            range: Range {
                                start: Position {
                                    line: line_idx as u32,
                                    character: char_start as u32,
                                },
                                end: Position {
                                    line: line_idx as u32,
                                    character: (char_start + word.len()) as u32,
                                },
                            },
                            kind: Some(DocumentHighlightKind::TEXT),
                        });
                        start = char_start + word.len();
                    }
                }

                return Ok(Some(highlights));
            }
        }

        Ok(None)
    }

    async fn code_action(
        &self,
        _params: CodeActionParams,
    ) -> LspResult<Option<CodeActionResponse>> {
        // TODO: Implement code actions (quick fixes, refactoring, etc.)
        // For now, return empty list
        Ok(Some(vec![]))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(VeyraLanguageServer::new);

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
