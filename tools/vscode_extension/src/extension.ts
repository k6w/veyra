import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
    Executable
} from 'vscode-languageclient/node';
import * as path from 'path';
import * as fs from 'fs';
import { exec } from 'child_process';
import { promisify } from 'util';
import { ToolManager } from './toolManager';

const execAsync = promisify(exec);

let client: LanguageClient;
let diagnosticCollection: vscode.DiagnosticCollection;
let compilerPath: string | null = null;
let outputChannel: vscode.OutputChannel;
let statusBarItem: vscode.StatusBarItem;
let errorStatusBarItem: vscode.StatusBarItem;
let runningStatusBarItem: vscode.StatusBarItem;
let toolManager: ToolManager;

export function activate(context: vscode.ExtensionContext) {
    console.log('Veyra Language Extension with Full IntelliSense is now active!');

    // Initialize output channel
    outputChannel = vscode.window.createOutputChannel('Veyra', { log: true });
    context.subscriptions.push(outputChannel);
    
    outputChannel.appendLine('═══════════════════════════════════════════════════');
    outputChannel.appendLine('Veyra Language Extension');
    outputChannel.appendLine('═══════════════════════════════════════════════════');
    outputChannel.appendLine('');

    // Initialize tool manager
    toolManager = new ToolManager(outputChannel);

    // Initialize diagnostic collection
    diagnosticCollection = vscode.languages.createDiagnosticCollection('veyra');
    context.subscriptions.push(diagnosticCollection);

    // Initialize status bar
    initializeStatusBar(context);

    // Find compiler path
    compilerPath = findVeyraCompiler();
    updateCompilerStatus();

    // Check tools availability
    toolManager.checkToolsAvailability().then(allAvailable => {
        if (!allAvailable) {
            outputChannel.appendLine('[Info] Some tools need to be built. Use "Veyra: Build Tools" command.');
        }
    });

    // Show welcome message
    showWelcomeMessage(context);

    // Register commands
    registerCommands(context);

    // Start enhanced language server
    startLanguageServer(context);

    // Setup real-time diagnostics
    setupDiagnostics(context);

    // Setup comprehensive IntelliSense
    setupIntelliSense(context);

    // Setup document formatting
    setupDocumentFormatting(context);

    // Setup auto-save actions
    setupAutoSaveActions(context);

    // Setup stdlib completion
    setupStdlibCompletion(context);

    // Setup hover information
    setupHoverProvider(context);

    // Setup signature help
    setupSignatureHelp(context);

    // Setup go to definition
    setupDefinitionProvider(context);

    // Setup symbol provider
    setupSymbolProvider(context);
    
    outputChannel.appendLine('[OK] Extension activated successfully!');
}

export function deactivate(): Thenable<void> | undefined {
    outputChannel.appendLine('Deactivating Veyra extension...');
    
    // Cleanup status bar items
    if (statusBarItem) {
        statusBarItem.dispose();
    }
    if (errorStatusBarItem) {
        errorStatusBarItem.dispose();
    }
    if (runningStatusBarItem) {
        runningStatusBarItem.dispose();
    }
    
    if (!client) {
        return undefined;
    }
    return client.stop();
}

// ===== UI INITIALIZATION =====

function initializeStatusBar(context: vscode.ExtensionContext) {
    // Main status bar item for Veyra
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    statusBarItem.command = 'veyra.showQuickActions';
    statusBarItem.tooltip = 'Click for Veyra quick actions';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);

    // Error status bar item
    errorStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 99);
    errorStatusBarItem.command = 'veyra.showProblems';
    errorStatusBarItem.tooltip = 'Click to view problems';
    errorStatusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
    context.subscriptions.push(errorStatusBarItem);

    // Running status bar item
    runningStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 98);
    runningStatusBarItem.tooltip = 'Veyra program running';
    context.subscriptions.push(runningStatusBarItem);

    // Update status bar when active editor changes
    context.subscriptions.push(
        vscode.window.onDidChangeActiveTextEditor(editor => {
            if (editor && editor.document.languageId === 'veyra') {
                updateStatusBar(editor.document);
            } else {
                statusBarItem.hide();
                errorStatusBarItem.hide();
            }
        })
    );

    // Initial update
    if (vscode.window.activeTextEditor?.document.languageId === 'veyra') {
        updateStatusBar(vscode.window.activeTextEditor.document);
    }
}

function updateStatusBar(document?: vscode.TextDocument) {
    if (!document || document.languageId !== 'veyra') {
        statusBarItem.hide();
        errorStatusBarItem.hide();
        return;
    }

    // Update main status
    const fileName = path.basename(document.fileName);
    statusBarItem.text = `$(file-code) Veyra: ${fileName}`;
    statusBarItem.show();

    // Update error status
    const diagnostics = vscode.languages.getDiagnostics(document.uri);
    if (diagnostics && diagnostics.length > 0) {
        const errors = diagnostics.filter(d => d.severity === vscode.DiagnosticSeverity.Error).length;
        const warnings = diagnostics.filter(d => d.severity === vscode.DiagnosticSeverity.Warning).length;
        
        if (errors > 0) {
            errorStatusBarItem.text = `$(error) ${errors} $(warning) ${warnings}`;
            errorStatusBarItem.show();
        } else if (warnings > 0) {
            errorStatusBarItem.text = `$(warning) ${warnings}`;
            errorStatusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
            errorStatusBarItem.show();
        } else {
            errorStatusBarItem.hide();
        }
    } else {
        errorStatusBarItem.hide();
    }
}

function updateCompilerStatus() {
    if (compilerPath && isExecutableAvailable(compilerPath)) {
        outputChannel.appendLine(`[OK] Veyra compiler found: ${compilerPath}`);
        vscode.window.showInformationMessage('Veyra compiler detected successfully!', { modal: false });
    } else {
        outputChannel.appendLine('[Warning] Veyra compiler not found. Some features may be limited.');
        outputChannel.appendLine('          Please install Veyra or configure the compiler path in settings.');
    }
}

function showWelcomeMessage(context: vscode.ExtensionContext) {
    const hasShownWelcome = context.globalState.get('veyra.hasShownWelcome', false);
    
    if (!hasShownWelcome) {
        const message = 'Welcome to Veyra! Get started by creating a new .vey file or opening an existing project.';
        vscode.window.showInformationMessage(message, 'Create New File', 'Open Folder', 'Don\'t Show Again')
            .then(selection => {
                if (selection === 'Create New File') {
                    vscode.commands.executeCommand('workbench.action.files.newUntitledFile', { languageId: 'veyra' });
                } else if (selection === 'Open Folder') {
                    vscode.commands.executeCommand('workbench.action.files.openFolder');
                } else if (selection === 'Don\'t Show Again') {
                    context.globalState.update('veyra.hasShownWelcome', true);
                }
            });
    }
}

// ===== COMMAND REGISTRATION =====

function registerCommands(context: vscode.ExtensionContext) {
    // Quick actions menu
    const quickActionsCommand = vscode.commands.registerCommand('veyra.showQuickActions', async () => {
        const actions = [
            { label: '$(play) Run Veyra File', description: 'Execute the current file', command: 'veyra.run' },
            { label: '$(tools) Build Project', description: 'Build the entire project', command: 'veyra.build' },
            { label: '$(symbol-keyword) Format File', description: 'Auto-format the current file', command: 'veyra.format' },
            { label: '$(checklist) Lint File', description: 'Check for code quality issues', command: 'veyra.lint' },
            { label: '$(new-folder) New Project', description: 'Create a new Veyra project', command: 'veyra.newProject' },
            { label: '$(gear) Build Veyra Tools', description: 'Compile formatter, linter, and package manager', command: 'veyra.buildTools' },
            { label: '$(output) Show Output', description: 'View Veyra output channel', command: 'veyra.showOutput' },
            { label: '$(book) Documentation', description: 'Open Veyra documentation', command: 'veyra.openDocs' }
        ];

        const selected = await vscode.window.showQuickPick(actions, {
            placeHolder: 'Select a Veyra action',
            matchOnDescription: true
        });

        if (selected) {
            vscode.commands.executeCommand(selected.command);
        }
    });

    // Build Veyra Tools command
    const buildToolsCommand = vscode.commands.registerCommand('veyra.buildTools', async () => {
        outputChannel.appendLine('');
        outputChannel.appendLine('═══════════════════════════════════════════════════');
        outputChannel.appendLine('Building Veyra Tools');
        outputChannel.appendLine('═══════════════════════════════════════════════════');
        
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('No workspace folder open. Please open the Veyra project folder.');
            return;
        }

        const toolsPath = path.join(workspaceFolder.uri.fsPath, 'tools');
        if (!fs.existsSync(toolsPath)) {
            vscode.window.showErrorMessage('Tools directory not found. Are you in the Veyra project?');
            return;
        }

        outputChannel.appendLine(`[Info] Building tools in: ${toolsPath}`);
        outputChannel.appendLine('[Info] This may take a few minutes...');
        outputChannel.show();

        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: 'Building Veyra tools (veyra-fmt, veyra-lint, veyra-pkg)...',
            cancellable: false
        }, async (progress) => {
            progress.report({ increment: 0 });
            
            const terminal = vscode.window.createTerminal({
                name: 'Build Veyra Tools',
                iconPath: new vscode.ThemeIcon('tools'),
                cwd: toolsPath
            });
            
            terminal.sendText('cargo build --release');
            terminal.show();
            
            outputChannel.appendLine('[OK] Build started in terminal');
            outputChannel.appendLine('[Info] Watch the terminal for build progress');
            outputChannel.appendLine('═══════════════════════════════════════════════════');
            
            progress.report({ increment: 100 });
            
            vscode.window.showInformationMessage(
                'Building Veyra tools. Check the terminal for progress.',
                'Show Terminal'
            ).then(selection => {
                if (selection === 'Show Terminal') {
                    terminal.show();
                }
            });
        });
    });

    // Show output channel
    const showOutputCommand = vscode.commands.registerCommand('veyra.showOutput', () => {
        outputChannel.show();
    });

    // Show problems panel
    const showProblemsCommand = vscode.commands.registerCommand('veyra.showProblems', () => {
        vscode.commands.executeCommand('workbench.actions.view.problems');
    });

    // Open documentation
    const openDocsCommand = vscode.commands.registerCommand('veyra.openDocs', () => {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (workspaceFolder) {
            const docsPath = path.join(workspaceFolder.uri.fsPath, 'docs');
            if (fs.existsSync(docsPath)) {
                vscode.commands.executeCommand('revealInExplorer', vscode.Uri.file(docsPath));
                return;
            }
        }
        vscode.window.showInformationMessage('📚 Veyra documentation is available in the docs folder of your project.');
    });

    // Run Veyra file with improved UI
    const runCommand = vscode.commands.registerCommand('veyra.run', async () => {
        outputChannel.appendLine('');
        outputChannel.appendLine('═══════════════════════════════════════════════════');
        outputChannel.appendLine('▶️  Running Veyra File');
        outputChannel.appendLine('═══════════════════════════════════════════════════');
        
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            const errorMsg = '❌ No Veyra file is open';
            outputChannel.appendLine(errorMsg);
            vscode.window.showErrorMessage(errorMsg);
            return;
        }

        const document = editor.document;
        if (document.languageId !== 'veyra') {
            const errorMsg = 'Current file is not a Veyra file (.vey)';
            outputChannel.appendLine('[Error] ' + errorMsg);
            vscode.window.showErrorMessage(errorMsg);
            return;
        }

        // Save first
        if (document.isDirty) {
            outputChannel.appendLine('[Info] Saving file...');
            await document.save();
        }

        const filePath = document.fileName;
        const fileName = path.basename(filePath);
        outputChannel.appendLine(`📄 File: ${fileName}`);

        const config = vscode.workspace.getConfiguration('veyra');
        const configuredPath = config.get('compilerPath', '') as string;
        const compiler = configuredPath || findVeyraCompiler();

        if (!compiler || !isExecutableAvailable(compiler)) {
            const errorMsg = 'Veyra compiler not found!\n\n' +
                'Please install the Veyra toolchain or configure the compiler path in settings.\n\n' +
                'Go to: File > Preferences > Settings > Veyra > Compiler Path';
            outputChannel.appendLine('[Error] ' + errorMsg.replace(/\n/g, ' '));
            
            vscode.window.showErrorMessage(
                'Veyra compiler not found',
                'Open Settings',
                'Install Veyra'
            ).then(selection => {
                if (selection === 'Open Settings') {
                    vscode.commands.executeCommand('workbench.action.openSettings', 'veyra.compilerPath');
                } else if (selection === 'Install Veyra') {
                    vscode.env.openExternal(vscode.Uri.parse('https://github.com/veyra-lang/veyra'));
                }
            });
            return;
        }

        outputChannel.appendLine(`[Info] Compiler: ${compiler}`);
        outputChannel.appendLine('');

        // Show running status
        runningStatusBarItem.text = '$(sync~spin) Running...';
        runningStatusBarItem.show();

        // Create terminal with nice branding
        const terminal = vscode.window.createTerminal({
            name: `Veyra Run: ${fileName}`,
            iconPath: new vscode.ThemeIcon('play')
        });
        
        terminal.sendText(`"${compiler}" "${filePath}"`);
        terminal.show();

        outputChannel.appendLine('[OK] Execution started in terminal');
        outputChannel.appendLine('═══════════════════════════════════════════════════');
        outputChannel.appendLine('');

        // Hide running status after a delay
        setTimeout(() => {
            runningStatusBarItem.hide();
        }, 3000);
    });

    // Build Veyra project with improved UI
    const buildCommand = vscode.commands.registerCommand('veyra.build', async () => {
        outputChannel.appendLine('');
        outputChannel.appendLine('═══════════════════════════════════════════════════');
        outputChannel.appendLine('Building Veyra Project');
        outputChannel.appendLine('═══════════════════════════════════════════════════');
        
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            const errorMsg = 'No workspace folder open';
            outputChannel.appendLine('[Error] ' + errorMsg);
            vscode.window.showErrorMessage(errorMsg);
            return;
        }

        // Check if veyra-pkg is available
        const pkgTool = await checkToolAvailability('veyra-pkg', 'Veyra Package Manager');
        if (!pkgTool) {
            return;
        }

        outputChannel.appendLine(`[Info] Workspace: ${workspaceFolder.name}`);
        outputChannel.appendLine(`[Info] Using veyra-pkg: ${pkgTool}`);
        
        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: 'Building Veyra project...',
            cancellable: false
        }, async (progress) => {
            progress.report({ increment: 0 });
            
            const terminal = vscode.window.createTerminal({
                name: 'Veyra Build',
                iconPath: new vscode.ThemeIcon('tools')
            });
            
            terminal.sendText(`cd "${workspaceFolder.uri.fsPath}"`);
            terminal.sendText(`"${pkgTool}" build`);
            terminal.show();
            
            outputChannel.appendLine('[OK] Build started');
            outputChannel.appendLine('═══════════════════════════════════════════════════');
            outputChannel.appendLine('');
            
            progress.report({ increment: 100 });
        });
    });

    // Format Veyra file with improved UI
    const formatCommand = vscode.commands.registerCommand('veyra.format', async () => {
        outputChannel.appendLine('');
        outputChannel.appendLine('═══════════════════════════════════════════════════');
        outputChannel.appendLine('Formatting Veyra File');
        outputChannel.appendLine('═══════════════════════════════════════════════════');
        
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            const errorMsg = 'No Veyra file is open';
            outputChannel.appendLine(errorMsg);
            vscode.window.showErrorMessage(errorMsg);
            return;
        }

        const document = editor.document;
        if (document.languageId !== 'veyra') {
            const errorMsg = '❌ Current file is not a Veyra file';
            outputChannel.appendLine(errorMsg);
            vscode.window.showErrorMessage(errorMsg);
            return;
        }

        await document.save();
        const filePath = document.fileName;
        const fileName = path.basename(filePath);
        outputChannel.appendLine(`[Info] File: ${fileName}`);

        // Check if veyra-fmt is available
        const fmtTool = await checkToolAvailability('veyra-fmt', 'Veyra Formatter');
        if (!fmtTool) {
            return;
        }

        outputChannel.appendLine(`[Info] Using veyra-fmt: ${fmtTool}`);

        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: `Formatting ${fileName}...`,
            cancellable: false
        }, async (progress) => {
            const terminal = vscode.window.createTerminal({
                name: 'Veyra Format',
                iconPath: new vscode.ThemeIcon('symbol-keyword')
            });
            terminal.sendText(`"${fmtTool}" --write "${filePath}"`);
            terminal.show();

            outputChannel.appendLine('[OK] Formatting completed');
            outputChannel.appendLine('═══════════════════════════════════════════════════');
            outputChannel.appendLine('');

            // Reload the file after formatting
            setTimeout(async () => {
                const uri = vscode.Uri.file(filePath);
                await vscode.workspace.openTextDocument(uri);
                vscode.window.showTextDocument(uri);
                vscode.window.showInformationMessage(`${fileName} formatted successfully!`);
            }, 1000);
        });
    });

    // Lint Veyra file with improved UI
    const lintCommand = vscode.commands.registerCommand('veyra.lint', async () => {
        outputChannel.appendLine('');
        outputChannel.appendLine('═══════════════════════════════════════════════════');
        outputChannel.appendLine('Linting Veyra File');
        outputChannel.appendLine('═══════════════════════════════════════════════════');
        
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            const errorMsg = '❌ No Veyra file is open';
            outputChannel.appendLine(errorMsg);
            vscode.window.showErrorMessage(errorMsg);
            return;
        }

        const document = editor.document;
        if (document.languageId !== 'veyra') {
            const errorMsg = '❌ Current file is not a Veyra file';
            outputChannel.appendLine(errorMsg);
            vscode.window.showErrorMessage(errorMsg);
            return;
        }

        await document.save();
        const filePath = document.fileName;
        const fileName = path.basename(filePath);
        outputChannel.appendLine(`[Info] File: ${fileName}`);

        // Check if veyra-lint is available
        const lintTool = await checkToolAvailability('veyra-lint', 'Veyra Linter');
        if (!lintTool) {
            return;
        }

        outputChannel.appendLine(`[Info] Using veyra-lint: ${lintTool}`);

        const terminal = vscode.window.createTerminal({
            name: 'Veyra Lint',
            iconPath: new vscode.ThemeIcon('checklist')
        });
        terminal.sendText(`"${lintTool}" --warnings "${filePath}"`);
        terminal.show();

        outputChannel.appendLine('[OK] Linting started');
        outputChannel.appendLine('═══════════════════════════════════════════════════');
        outputChannel.appendLine('');
    });

    // New Veyra project with improved UI
    const newProjectCommand = vscode.commands.registerCommand('veyra.newProject', async () => {
        outputChannel.appendLine('');
        outputChannel.appendLine('═══════════════════════════════════════════════════');
        outputChannel.appendLine('Creating New Veyra Project');
        outputChannel.appendLine('═══════════════════════════════════════════════════');
        
        const projectName = await vscode.window.showInputBox({
            prompt: 'Enter project name',
            placeHolder: 'my-veyra-project',
            validateInput: (value) => {
                if (!value) {
                    return 'Project name cannot be empty';
                }
                if (!/^[a-zA-Z0-9_-]+$/.test(value)) {
                    return 'Project name can only contain letters, numbers, hyphens, and underscores';
                }
                return null;
            }
        });

        if (!projectName) {
            outputChannel.appendLine('[Info] Project creation cancelled');
            return;
        }

        outputChannel.appendLine(`[Info] Project name: ${projectName}`);

        // Check if veyra-pkg is available
        const pkgTool = await checkToolAvailability('veyra-pkg', 'Veyra Package Manager');
        if (!pkgTool) {
            return;
        }

        outputChannel.appendLine(`[Info] Using veyra-pkg: ${pkgTool}`);

        const folderUri = await vscode.window.showOpenDialog({
            canSelectFolders: true,
            canSelectFiles: false,
            canSelectMany: false,
            openLabel: 'Select Project Location',
            title: 'Choose where to create your Veyra project'
        });

        if (!folderUri || folderUri.length === 0) {
            outputChannel.appendLine('[Info] Project creation cancelled');
            return;
        }

        const projectPath = path.join(folderUri[0].fsPath, projectName);
        outputChannel.appendLine(`[Info] Location: ${projectPath}`);

        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: `Creating project '${projectName}'...`,
            cancellable: false
        }, async (progress) => {
            const terminal = vscode.window.createTerminal({
                name: 'Veyra New Project',
                iconPath: new vscode.ThemeIcon('new-folder')
            });
            terminal.sendText(`"${pkgTool}" init ${projectName} --path "${projectPath}"`);
            terminal.show();

            outputChannel.appendLine('[OK] Project creation started');
            outputChannel.appendLine('═══════════════════════════════════════════════════');
            outputChannel.appendLine('');

            // Ask to open the new project
            setTimeout(async () => {
                const openProject = await vscode.window.showInformationMessage(
                    `Project '${projectName}' created successfully! Open it now?`,
                    { modal: true },
                    'Open Project', 'Not Now'
                );

                if (openProject === 'Open Project') {
                    const uri = vscode.Uri.file(projectPath);
                    await vscode.commands.executeCommand('vscode.openFolder', uri);
                }
            }, 2000);
        });
    });

    // Register all commands
    context.subscriptions.push(
        quickActionsCommand,
        buildToolsCommand,
        showOutputCommand,
        showProblemsCommand,
        openDocsCommand,
        runCommand,
        buildCommand,
        formatCommand,
        lintCommand,
        newProjectCommand
    );
}

// ===== DIAGNOSTIC SYSTEM =====

function setupDiagnostics(context: vscode.ExtensionContext) {
    // Validate on open
    context.subscriptions.push(
        vscode.workspace.onDidOpenTextDocument(document => {
            if (document.languageId === 'veyra') {
                validateDocument(document);
            }
        })
    );

    // Validate on change (with debounce)
    let timeoutId: NodeJS.Timeout | undefined;
    context.subscriptions.push(
        vscode.workspace.onDidChangeTextDocument(event => {
            if (event.document.languageId === 'veyra') {
                if (timeoutId) {
                    clearTimeout(timeoutId);
                }
                timeoutId = setTimeout(() => {
                    validateDocument(event.document);
                }, 500); // 500ms debounce
            }
        })
    );

    // Validate on save
    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument(document => {
            if (document.languageId === 'veyra') {
                validateDocument(document);
            }
        })
    );

    // Clear diagnostics on close
    context.subscriptions.push(
        vscode.workspace.onDidCloseTextDocument(document => {
            if (document.languageId === 'veyra') {
                diagnosticCollection.delete(document.uri);
            }
        })
    );

    // Validate all open Veyra files
    vscode.workspace.textDocuments.forEach(document => {
        if (document.languageId === 'veyra') {
            validateDocument(document);
        }
    });
}

async function validateDocument(document: vscode.TextDocument) {
    if (!compilerPath) {
        // If no compiler found, clear diagnostics
        diagnosticCollection.delete(document.uri);
        updateStatusBar(document);
        return;
    }

    try {
        // Save document content to temp file
        const tempFilePath = document.uri.fsPath;
        
        // Run compiler in check mode (syntax check only)
        const { stdout, stderr } = await execAsync(
            `"${compilerPath}" "${tempFilePath}"`,
            { timeout: 5000 }
        );

        // If successful, clear diagnostics
        diagnosticCollection.delete(document.uri);
        updateStatusBar(document);

    } catch (error: any) {
        // Parse error output
        const diagnostics = parseCompilerErrors(error.stderr || error.stdout || error.message, document);
        diagnosticCollection.set(document.uri, diagnostics);
        updateStatusBar(document);
    }
}

function parseCompilerErrors(errorOutput: string, document: vscode.TextDocument): vscode.Diagnostic[] {
    const diagnostics: vscode.Diagnostic[] = [];
    
    console.log('Raw compiler output:', errorOutput);
    
    // Match patterns like:
    // "Lexer error at line 135, column 38: Unexpected character '&'"
    // "Parser error at line 5, column 13: Expected expression"
    // "Error: ParseError { line: 15, column: 1, message: "Expected '}' after block, found ''" }"
    // "Error: LexError { line: 5, column: 10, message: "Unexpected character: @" }"
    // "Syntax Error at line 3, column 5: Unexpected token"
    
    const patterns = [
        // Pattern 1: "Lexer/Parser error at line X, column Y: message" (most common)
        /(?:Lexer|Parser|Syntax|Parse)\s+error\s+at\s+line\s+(\d+),\s+column\s+(\d+):\s+(.+?)(?:\n|$)/gi,
        // Pattern 2: ParseError/LexError with struct format
        /(?:ParseError|LexError)\s*{\s*line:\s*(\d+),\s*column:\s*(\d+),\s*message:\s*"([^"]+)"/g,
        // Pattern 3: Simple "Error at line X, column Y: message"
        /Error\s+at\s+line\s+(\d+),\s+column\s+(\d+):\s+(.+?)(?:\n|$)/gi,
        // Pattern 4: Generic error format
        /error:\s*line\s+(\d+),\s+column\s+(\d+):\s*(.+?)(?:\n|$)/gi,
    ];

    for (const pattern of patterns) {
        let match;
        pattern.lastIndex = 0; // Reset regex state
        while ((match = pattern.exec(errorOutput)) !== null) {
            const line = parseInt(match[1]) - 1; // Convert to 0-based
            const column = parseInt(match[2]) - 1; // Convert to 0-based
            const message = match[3].trim();

            console.log(`Parsed error: line ${line + 1}, column ${column + 1}, message: ${message}`);

            // Ensure line and column are within bounds
            const validLine = Math.max(0, Math.min(line, document.lineCount - 1));
            const lineText = document.lineAt(validLine).text;
            const validColumn = Math.max(0, Math.min(column, lineText.length));

            // Create range for the error
            let range: vscode.Range;
            if (validColumn < lineText.length) {
                // Try to highlight the problematic token
                const endColumn = findTokenEnd(lineText, validColumn);
                range = new vscode.Range(validLine, validColumn, validLine, endColumn);
            } else {
                // If column is at end of line, highlight the whole line
                range = new vscode.Range(validLine, 0, validLine, lineText.length);
            }

            // Determine severity based on error type
            let severity = vscode.DiagnosticSeverity.Error;
            if (message.toLowerCase().includes('warning')) {
                severity = vscode.DiagnosticSeverity.Warning;
            }

            const diagnostic = new vscode.Diagnostic(
                range,
                message,
                severity
            );
            diagnostic.source = 'veyra';
            
            diagnostics.push(diagnostic);
        }
    }

    if (diagnostics.length === 0) {
        console.log('No diagnostics parsed from error output');
    }

    return diagnostics;
}

function findTokenEnd(line: string, start: number): number {
    // Find the end of the current token
    let end = start;
    
    // Skip whitespace
    while (end < line.length && /\s/.test(line[end])) {
        end++;
    }
    
    // If we hit a delimiter, highlight just that character
    if (end < line.length && /[{}()\[\];,.]/.test(line[end])) {
        return end + 1;
    }
    
    // Otherwise, find the end of the word/token
    while (end < line.length && /[a-zA-Z0-9_]/.test(line[end])) {
        end++;
    }
    
    return Math.max(end, start + 1);
}

// ===== LANGUAGE SERVER =====

function startLanguageServer(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('veyra');
    const configuredPath = config.get('languageServerPath', '') as string;
    
    // Auto-detect language server path
    const serverPath = configuredPath || findVeyraLanguageServer();

    // Check if language server is available
    if (!serverPath || !isExecutableAvailable(serverPath)) {
        console.log('Veyra Language Server not found. Using built-in diagnostics.');
        return;
    }

    const serverOptions: ServerOptions = {
        command: serverPath as string,
        transport: TransportKind.stdio
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'veyra' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.vey')
        }
    };

    client = new LanguageClient(
        'veyraLanguageServer',
        'Veyra Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}

function setupDocumentFormatting(context: vscode.ExtensionContext) {
    const formatProvider = vscode.languages.registerDocumentFormattingEditProvider(
        'veyra',
        {
            provideDocumentFormattingEdits(document: vscode.TextDocument): vscode.TextEdit[] {
                // This will be handled by the language server
                // or we can implement a direct formatter here
                return [];
            }
        }
    );

    context.subscriptions.push(formatProvider);
}

function setupAutoSaveActions(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('veyra');

    if (config.get('formatOnSave', true)) {
        const formatOnSave = vscode.workspace.onWillSaveTextDocument(event => {
            if (event.document.languageId === 'veyra') {
                event.waitUntil(
                    vscode.commands.executeCommand('editor.action.formatDocument')
                );
            }
        });
        context.subscriptions.push(formatOnSave);
    }

    if (config.get('lintOnSave', true)) {
        const lintOnSave = vscode.workspace.onDidSaveTextDocument(document => {
            if (document.languageId === 'veyra') {
                // Trigger linting through language server diagnostics
                // This happens automatically with the language server
            }
        });
        context.subscriptions.push(lintOnSave);
    }
}

function findVeyraLanguageServer(): string | null {
    const { execSync } = require('child_process');
    
    // Try common locations
    const possiblePaths = [
        'veyra-lsp',  // From PATH
        'veyra-lsp.exe',  // Windows from PATH
    ];
    
    // Check workspace for Veyra project structure
    if (vscode.workspace.workspaceFolders) {
        for (const folder of vscode.workspace.workspaceFolders) {
            const workspacePath = folder.uri.fsPath;
            
            // Check if this looks like a Veyra project directory
            possiblePaths.push(
                path.join(workspacePath, 'tools', 'target', 'release', 'veyra-lsp.exe'),
                path.join(workspacePath, 'tools', 'target', 'debug', 'veyra-lsp.exe'),
                path.join(workspacePath, 'tools', 'lsp', 'target', 'release', 'veyra-lsp.exe'),
                path.join(workspacePath, 'tools', 'lsp', 'target', 'debug', 'veyra-lsp.exe'),
                path.join(workspacePath, 'target', 'release', 'veyra-lsp.exe'),
                path.join(workspacePath, 'target', 'debug', 'veyra-lsp.exe')
            );
            
            // Check parent directories in case we're in a subdirectory
            let parentPath = path.dirname(workspacePath);
            for (let i = 0; i < 3; i++) {
                possiblePaths.push(
                    path.join(parentPath, 'tools', 'target', 'release', 'veyra-lsp.exe'),
                    path.join(parentPath, 'tools', 'target', 'debug', 'veyra-lsp.exe'),
                    path.join(parentPath, 'tools', 'lsp', 'target', 'release', 'veyra-lsp.exe'),
                    path.join(parentPath, 'tools', 'lsp', 'target', 'debug', 'veyra-lsp.exe')
                );
                parentPath = path.dirname(parentPath);
            }
        }
    }
    
    // Test each possible path
    for (const testPath of possiblePaths) {
        if (isExecutableAvailable(testPath)) {
            return testPath;
        }
    }
    
    return null;
}

function findVeyraCompiler(): string | null {
    const possiblePaths = [
        'veyc',  // From PATH
        'veyc.exe',  // Windows from PATH
    ];
    
    // Check workspace for Veyra project structure
    if (vscode.workspace.workspaceFolders) {
        for (const folder of vscode.workspace.workspaceFolders) {
            const workspacePath = folder.uri.fsPath;
            
            possiblePaths.push(
                path.join(workspacePath, 'compiler', 'target', 'release', 'veyc.exe'),
                path.join(workspacePath, 'compiler', 'target', 'debug', 'veyc.exe'),
                path.join(workspacePath, 'target', 'release', 'veyc.exe'),
                path.join(workspacePath, 'target', 'debug', 'veyc.exe')
            );
            
            // Check parent directories
            let parentPath = path.dirname(workspacePath);
            for (let i = 0; i < 3; i++) {
                possiblePaths.push(
                    path.join(parentPath, 'compiler', 'target', 'release', 'veyc.exe'),
                    path.join(parentPath, 'compiler', 'target', 'debug', 'veyc.exe')
                );
                parentPath = path.dirname(parentPath);
            }
        }
    }
    
    // Test each possible path
    for (const testPath of possiblePaths) {
        if (isExecutableAvailable(testPath)) {
            return testPath;
        }
    }
    
    return null;
}

function isExecutableAvailable(executablePath: string): boolean {
    try {
        const { execSync } = require('child_process');
        
        // Check if file exists first (for absolute paths)
        if (path.isAbsolute(executablePath) && !fs.existsSync(executablePath)) {
            return false;
        }
        
        execSync(`"${executablePath}" --help`, { stdio: 'ignore', timeout: 5000 });
        return true;
    } catch (error) {
        return false;
    }
}

// ===== TOOL DISCOVERY FUNCTIONS =====

function findVeyraTool(toolName: string): string | null {
    const possiblePaths = [
        toolName,  // From PATH
        `${toolName}.exe`,  // Windows from PATH
    ];
    
    // Check workspace for tool
    if (vscode.workspace.workspaceFolders) {
        for (const folder of vscode.workspace.workspaceFolders) {
            const workspacePath = folder.uri.fsPath;
            
            possiblePaths.push(
                path.join(workspacePath, 'tools', 'target', 'release', `${toolName}.exe`),
                path.join(workspacePath, 'tools', 'target', 'debug', `${toolName}.exe`),
                path.join(workspacePath, 'target', 'release', `${toolName}.exe`),
                path.join(workspacePath, 'target', 'debug', `${toolName}.exe`)
            );
            
            // Check parent directories
            let parentPath = path.dirname(workspacePath);
            for (let i = 0; i < 3; i++) {
                possiblePaths.push(
                    path.join(parentPath, 'tools', 'target', 'release', `${toolName}.exe`),
                    path.join(parentPath, 'tools', 'target', 'debug', `${toolName}.exe`)
                );
                parentPath = path.dirname(parentPath);
            }
        }
    }
    
    // Check configuration
    const config = vscode.workspace.getConfiguration('veyra');
    const configuredPath = config.get(`${toolName}Path`, '') as string;
    if (configuredPath) {
        possiblePaths.unshift(configuredPath);
    }
    
    // Test each possible path
    for (const testPath of possiblePaths) {
        if (isExecutableAvailable(testPath)) {
            return testPath;
        }
    }
    
    return null;
}

async function checkToolAvailability(toolName: string, friendlyName: string): Promise<string | null> {
    const tool = findVeyraTool(toolName);
    
    if (!tool) {
        const selection = await vscode.window.showWarningMessage(
            `${friendlyName} not found. Would you like to build the Veyra tools?`,
            'Build Tools',
            'Configure Path',
            'Continue Anyway'
        );
        
        if (selection === 'Build Tools') {
            vscode.commands.executeCommand('veyra.buildTools');
            return null;
        } else if (selection === 'Configure Path') {
            vscode.commands.executeCommand('workbench.action.openSettings', `veyra.${toolName}Path`);
            return null;
        }
    }
    
    return tool;
}

// ===== ENHANCED INTELLISENSE IMPLEMENTATION =====

function setupIntelliSense(context: vscode.ExtensionContext) {
    // Core IntelliSense features are handled by the language server
    // This function sets up additional client-side features
    
    // Add workspace symbols refresh on file changes
    const refreshSymbols = vscode.commands.registerCommand('veyra.refreshSymbols', () => {
        if (client) {
            client.sendNotification('workspace/didChangeWatchedFiles', {
                changes: [{
                    uri: vscode.window.activeTextEditor?.document.uri.toString(),
                    type: 2 // Changed
                }]
            });
        }
    });
    
    context.subscriptions.push(refreshSymbols);
}

function setupStdlibCompletion(context: vscode.ExtensionContext) {
    // Register completion provider for stdlib functions
    const completionProvider = vscode.languages.registerCompletionItemProvider(
        'veyra',
        {
            provideCompletionItems(document: vscode.TextDocument, position: vscode.Position): vscode.CompletionItem[] {
                const items: vscode.CompletionItem[] = [];
                
                // Core functions
                items.push(...getCoreCompletions());
                
                // Math functions
                items.push(...getMathCompletions());
                
                // String functions
                items.push(...getStringCompletions());
                
                // Collections functions
                items.push(...getCollectionsCompletions());
                
                // IO functions
                items.push(...getIOCompletions());
                
                // Network functions
                items.push(...getNetworkCompletions());
                
                // DateTime functions
                items.push(...getDateTimeCompletions());
                
                return items;
            }
        },
        '.', // Trigger on dot
        ':', // Trigger on colon (for module access)
        '(' // Trigger on opening parenthesis
    );
    
    context.subscriptions.push(completionProvider);
}

function setupHoverProvider(context: vscode.ExtensionContext) {
    const hoverProvider = vscode.languages.registerHoverProvider('veyra', {
        provideHover(document: vscode.TextDocument, position: vscode.Position): vscode.Hover | undefined {
            const wordRange = document.getWordRangeAtPosition(position);
            if (!wordRange) return undefined;
            
            const word = document.getText(wordRange);
            const hoverInfo = getHoverInfo(word);
            
            if (hoverInfo) {
                return new vscode.Hover(new vscode.MarkdownString(hoverInfo));
            }
            
            return undefined;
        }
    });
    
    context.subscriptions.push(hoverProvider);
}

function setupSignatureHelp(context: vscode.ExtensionContext) {
    const signatureProvider = vscode.languages.registerSignatureHelpProvider(
        'veyra', 
        {
            provideSignatureHelp(document: vscode.TextDocument, position: vscode.Position): vscode.SignatureHelp | undefined {
                const line = document.lineAt(position).text;
                const beforeCursor = line.substring(0, position.character);
                
                // Find function call pattern
                const functionMatch = beforeCursor.match(/(\w+)\s*\(/);
                if (functionMatch) {
                    const functionName = functionMatch[1];
                    const signature = getFunctionSignature(functionName);
                    
                    if (signature) {
                        const help = new vscode.SignatureHelp();
                        help.signatures = [signature];
                        help.activeSignature = 0;
                        help.activeParameter = getActiveParameter(beforeCursor);
                        return help;
                    }
                }
                
                return undefined;
            }
        },
        '(', ','
    );
    
    context.subscriptions.push(signatureProvider);
}

function setupDefinitionProvider(context: vscode.ExtensionContext) {
    const definitionProvider = vscode.languages.registerDefinitionProvider('veyra', {
        provideDefinition(document: vscode.TextDocument, position: vscode.Position): vscode.Location[] | undefined {
            const wordRange = document.getWordRangeAtPosition(position);
            if (!wordRange) return undefined;
            
            const word = document.getText(wordRange);
            
            // Check if it's a stdlib function
            const stdlibLocation = getStdlibLocation(word);
            if (stdlibLocation) {
                return [stdlibLocation];
            }
            
            // For user-defined functions, the language server handles this
            return undefined;
        }
    });
    
    context.subscriptions.push(definitionProvider);
}

function setupSymbolProvider(context: vscode.ExtensionContext) {
    // Document symbol provider
    const documentSymbolProvider = vscode.languages.registerDocumentSymbolProvider('veyra', {
        provideDocumentSymbols(document: vscode.TextDocument): vscode.DocumentSymbol[] {
            const symbols: vscode.DocumentSymbol[] = [];
            const text = document.getText();
            const lines = text.split('\n');
            
            for (let i = 0; i < lines.length; i++) {
                const line = lines[i];
                
                // Function definitions
                const funcMatch = line.match(/^\s*fn\s+(\w+)\s*\(/);
                if (funcMatch) {
                    const name = funcMatch[1];
                    const range = new vscode.Range(i, 0, i, line.length);
                    const symbol = new vscode.DocumentSymbol(
                        name,
                        '',
                        vscode.SymbolKind.Function,
                        range,
                        range
                    );
                    symbols.push(symbol);
                }
                
                // Variable definitions
                const varMatch = line.match(/^\s*let\s+(\w+)/);
                if (varMatch) {
                    const name = varMatch[1];
                    const range = new vscode.Range(i, 0, i, line.length);
                    const symbol = new vscode.DocumentSymbol(
                        name,
                        '',
                        vscode.SymbolKind.Variable,
                        range,
                        range
                    );
                    symbols.push(symbol);
                }
            }
            
            return symbols;
        }
    });
    
    context.subscriptions.push(documentSymbolProvider);
}

// ===== COMPLETION ITEM GENERATORS =====

function getCoreCompletions(): vscode.CompletionItem[] {
    const items: vscode.CompletionItem[] = [];
    
    const coreFunctions = [
        { name: 'is_int', params: 'value', desc: 'Check if value is integer' },
        { name: 'is_float', params: 'value', desc: 'Check if value is float' },
        { name: 'is_string', params: 'value', desc: 'Check if value is string' },
        { name: 'is_bool', params: 'value', desc: 'Check if value is boolean' },
        { name: 'is_array', params: 'value', desc: 'Check if value is array' },
        { name: 'is_none', params: 'value', desc: 'Check if value is None' },
        { name: 'type_of', params: 'value', desc: 'Get type name as string' },
        { name: 'to_int', params: 'value', desc: 'Convert value to integer' },
        { name: 'to_float', params: 'value', desc: 'Convert value to float' },
        { name: 'to_string', params: 'value', desc: 'Convert value to string' },
        { name: 'to_bool', params: 'value', desc: 'Convert value to boolean' },
        { name: 'len', params: 'value', desc: 'Get length of array or string' },
        { name: 'print', params: 'message', desc: 'Print message to stdout' },
        { name: 'deep_copy', params: 'value', desc: 'Create deep copy of value' }
    ];
    
    for (const func of coreFunctions) {
        const item = new vscode.CompletionItem(func.name, vscode.CompletionItemKind.Function);
        item.detail = func.desc;
        item.insertText = new vscode.SnippetString(`${func.name}(\${1:${func.params}})`);
        item.documentation = new vscode.MarkdownString(`**${func.name}(${func.params})**\n\n${func.desc}`);
        items.push(item);
    }
    
    return items;
}

function getMathCompletions(): vscode.CompletionItem[] {
    const items: vscode.CompletionItem[] = [];
    
    // Math constants
    const constants = [
        { name: 'PI', desc: 'Mathematical constant π (3.14159...)' },
        { name: 'E', desc: 'Mathematical constant e (2.71828...)' },
        { name: 'PHI', desc: 'Golden ratio φ (1.61803...)' },
        { name: 'TAU', desc: 'Mathematical constant τ = 2π' }
    ];
    
    for (const constant of constants) {
        const item = new vscode.CompletionItem(constant.name, vscode.CompletionItemKind.Constant);
        item.detail = constant.desc;
        item.documentation = new vscode.MarkdownString(`**${constant.name}**\n\n${constant.desc}`);
        items.push(item);
    }
    
    // Math functions
    const mathFunctions = [
        { name: 'abs', params: 'x', desc: 'Absolute value' },
        { name: 'sign', params: 'x', desc: 'Sign function (-1, 0, 1)' },
        { name: 'min', params: 'a, b', desc: 'Minimum of two values' },
        { name: 'max', params: 'a, b', desc: 'Maximum of two values' },
        { name: 'clamp', params: 'value, min_val, max_val', desc: 'Clamp value between min and max' },
        { name: 'pow', params: 'base, exponent', desc: 'Power function (base^exponent)' },
        { name: 'sqrt', params: 'x', desc: 'Square root' },
        { name: 'cbrt', params: 'x', desc: 'Cube root' },
        { name: 'exp', params: 'x', desc: 'Exponential function (e^x)' },
        { name: 'ln', params: 'x', desc: 'Natural logarithm' },
        { name: 'log10', params: 'x', desc: 'Base-10 logarithm' },
        { name: 'log2', params: 'x', desc: 'Base-2 logarithm' },
        { name: 'sin', params: 'x', desc: 'Sine function' },
        { name: 'cos', params: 'x', desc: 'Cosine function' },
        { name: 'tan', params: 'x', desc: 'Tangent function' },
        { name: 'asin', params: 'x', desc: 'Arcsine function' },
        { name: 'acos', params: 'x', desc: 'Arccosine function' },
        { name: 'atan', params: 'x', desc: 'Arctangent function' },
        { name: 'sinh', params: 'x', desc: 'Hyperbolic sine' },
        { name: 'cosh', params: 'x', desc: 'Hyperbolic cosine' },
        { name: 'tanh', params: 'x', desc: 'Hyperbolic tangent' },
        { name: 'factorial', params: 'n', desc: 'Factorial function' },
        { name: 'binomial', params: 'n, k', desc: 'Binomial coefficient C(n,k)' },
        { name: 'permutation', params: 'n, k', desc: 'Permutation P(n,k)' },
        { name: 'floor', params: 'x', desc: 'Floor function' },
        { name: 'ceil', params: 'x', desc: 'Ceiling function' },
        { name: 'round', params: 'x', desc: 'Round to nearest integer' },
        { name: 'mean', params: 'values', desc: 'Calculate mean of array' },
        { name: 'variance', params: 'values', desc: 'Calculate variance of array' },
        { name: 'std_dev', params: 'values', desc: 'Calculate standard deviation' }
    ];
    
    for (const func of mathFunctions) {
        const item = new vscode.CompletionItem(func.name, vscode.CompletionItemKind.Function);
        item.detail = func.desc;
        item.insertText = new vscode.SnippetString(`${func.name}(\${1:${func.params}})`);
        item.documentation = new vscode.MarkdownString(`**${func.name}(${func.params})**\n\n${func.desc}`);
        items.push(item);
    }
    
    return items;
}

function getStringCompletions(): vscode.CompletionItem[] {
    const items: vscode.CompletionItem[] = [];
    
    const stringFunctions = [
        { name: 'string_length', params: 'str', desc: 'Get string length' },
        { name: 'string_concat', params: 'str1, str2', desc: 'Concatenate two strings' },
        { name: 'string_substring', params: 'str, start, length', desc: 'Extract substring' },
        { name: 'string_index_of', params: 'str, substr', desc: 'Find substring index' },
        { name: 'string_last_index_of', params: 'str, substr', desc: 'Find last substring index' },
        { name: 'string_starts_with', params: 'str, prefix', desc: 'Check if string starts with prefix' },
        { name: 'string_ends_with', params: 'str, suffix', desc: 'Check if string ends with suffix' },
        { name: 'string_contains', params: 'str, substr', desc: 'Check if string contains substring' },
        { name: 'string_to_upper', params: 'str', desc: 'Convert to uppercase' },
        { name: 'string_to_lower', params: 'str', desc: 'Convert to lowercase' },
        { name: 'string_trim', params: 'str', desc: 'Remove leading/trailing whitespace' },
        { name: 'string_trim_start', params: 'str', desc: 'Remove leading whitespace' },
        { name: 'string_trim_end', params: 'str', desc: 'Remove trailing whitespace' },
        { name: 'string_replace', params: 'str, old, new', desc: 'Replace all occurrences' },
        { name: 'string_replace_first', params: 'str, old, new', desc: 'Replace first occurrence' },
        { name: 'string_split', params: 'str, delimiter', desc: 'Split string into array' },
        { name: 'string_join', params: 'array, separator', desc: 'Join array elements with separator' },
        { name: 'string_repeat', params: 'str, count', desc: 'Repeat string count times' },
        { name: 'string_reverse', params: 'str', desc: 'Reverse string' },
        { name: 'string_pad_left', params: 'str, length, pad_char', desc: 'Pad string on left' },
        { name: 'string_pad_right', params: 'str, length, pad_char', desc: 'Pad string on right' },
        { name: 'string_is_numeric', params: 'str', desc: 'Check if string is numeric' },
        { name: 'string_is_alpha', params: 'str', desc: 'Check if string is alphabetic' },
        { name: 'string_is_alphanumeric', params: 'str', desc: 'Check if string is alphanumeric' },
        { name: 'string_is_whitespace', params: 'str', desc: 'Check if string is whitespace' }
    ];
    
    for (const func of stringFunctions) {
        const item = new vscode.CompletionItem(func.name, vscode.CompletionItemKind.Function);
        item.detail = func.desc;
        item.insertText = new vscode.SnippetString(`${func.name}(\${1:${func.params}})`);
        item.documentation = new vscode.MarkdownString(`**${func.name}(${func.params})**\n\n${func.desc}`);
        items.push(item);
    }
    
    return items;
}

function getCollectionsCompletions(): vscode.CompletionItem[] {
    const items: vscode.CompletionItem[] = [];
    
    const collectionFunctions = [
        { name: 'array_push', params: 'array, element', desc: 'Add element to end of array' },
        { name: 'array_pop', params: 'array', desc: 'Remove and return last element' },
        { name: 'array_unshift', params: 'array, element', desc: 'Add element to beginning' },
        { name: 'array_shift', params: 'array', desc: 'Remove and return first element' },
        { name: 'array_insert', params: 'array, index, element', desc: 'Insert element at index' },
        { name: 'array_remove_at', params: 'array, index', desc: 'Remove element at index' },
        { name: 'array_remove', params: 'array, element', desc: 'Remove first occurrence of element' },
        { name: 'array_clear', params: 'array', desc: 'Remove all elements from array' },
        { name: 'array_contains', params: 'array, element', desc: 'Check if array contains element' },
        { name: 'array_index_of', params: 'array, element', desc: 'Find index of element' },
        { name: 'array_last_index_of', params: 'array, element', desc: 'Find last index of element' },
        { name: 'array_slice', params: 'array, start, end', desc: 'Extract slice of array' },
        { name: 'array_concat', params: 'array1, array2', desc: 'Concatenate two arrays' },
        { name: 'array_reverse', params: 'array', desc: 'Reverse array elements' },
        { name: 'array_sort', params: 'array', desc: 'Sort array elements' },
        { name: 'array_sort_by', params: 'array, compareFn', desc: 'Sort with custom compare function' },
        { name: 'array_map', params: 'array, mapFn', desc: 'Transform each element' },
        { name: 'array_filter', params: 'array, filterFn', desc: 'Filter elements by predicate' },
        { name: 'array_reduce', params: 'array, reduceFn, initial', desc: 'Reduce array to single value' },
        { name: 'array_find', params: 'array, predicate', desc: 'Find first matching element' },
        { name: 'array_find_index', params: 'array, predicate', desc: 'Find index of first match' },
        { name: 'array_every', params: 'array, predicate', desc: 'Check if all elements match' },
        { name: 'array_some', params: 'array, predicate', desc: 'Check if any element matches' },
        { name: 'array_unique', params: 'array', desc: 'Remove duplicate elements' },
        { name: 'array_flatten', params: 'array', desc: 'Flatten nested arrays' },
        { name: 'array_chunk', params: 'array, size', desc: 'Split array into chunks' },
        { name: 'array_zip', params: 'array1, array2', desc: 'Zip two arrays together' },
        { name: 'array_min', params: 'array', desc: 'Find minimum value' },
        { name: 'array_max', params: 'array', desc: 'Find maximum value' },
        { name: 'array_sum', params: 'array', desc: 'Sum all numeric elements' }
    ];
    
    for (const func of collectionFunctions) {
        const item = new vscode.CompletionItem(func.name, vscode.CompletionItemKind.Function);
        item.detail = func.desc;
        item.insertText = new vscode.SnippetString(`${func.name}(\${1:${func.params}})`);
        item.documentation = new vscode.MarkdownString(`**${func.name}(${func.params})**\n\n${func.desc}`);
        items.push(item);
    }
    
    return items;
}

function getIOCompletions(): vscode.CompletionItem[] {
    const items: vscode.CompletionItem[] = [];
    
    const ioFunctions = [
        { name: 'read_file', params: 'filename', desc: 'Read entire file as string' },
        { name: 'write_file', params: 'filename, content', desc: 'Write string to file' },
        { name: 'append_file', params: 'filename, content', desc: 'Append string to file' },
        { name: 'read_lines', params: 'filename', desc: 'Read file as array of lines' },
        { name: 'write_lines', params: 'filename, lines', desc: 'Write array of lines to file' },
        { name: 'file_exists', params: 'filename', desc: 'Check if file exists' },
        { name: 'file_size', params: 'filename', desc: 'Get file size in bytes' },
        { name: 'file_modified_time', params: 'filename', desc: 'Get file modification timestamp' },
        { name: 'delete_file', params: 'filename', desc: 'Delete a file' },
        { name: 'copy_file', params: 'source, destination', desc: 'Copy file to new location' },
        { name: 'move_file', params: 'source, destination', desc: 'Move/rename file' },
        { name: 'create_directory', params: 'dirname', desc: 'Create directory (recursive)' },
        { name: 'remove_directory', params: 'dirname', desc: 'Remove directory (recursive)' },
        { name: 'directory_exists', params: 'dirname', desc: 'Check if directory exists' },
        { name: 'list_directory', params: 'dirname', desc: 'List directory contents' },
        { name: 'list_files', params: 'dirname', desc: 'List files in directory' },
        { name: 'list_directories', params: 'dirname', desc: 'List subdirectories' },
        { name: 'current_directory', params: '', desc: 'Get current working directory' },
        { name: 'change_directory', params: 'dirname', desc: 'Change working directory' },
        { name: 'absolute_path', params: 'path', desc: 'Convert to absolute path' },
        { name: 'relative_path', params: 'path, base', desc: 'Get relative path' },
        { name: 'path_join', params: 'parts...', desc: 'Join path components' },
        { name: 'path_dirname', params: 'path', desc: 'Get directory name' },
        { name: 'path_basename', params: 'path', desc: 'Get file name' },
        { name: 'path_extension', params: 'path', desc: 'Get file extension' },
        { name: 'path_stem', params: 'path', desc: 'Get filename without extension' }
    ];
    
    for (const func of ioFunctions) {
        const item = new vscode.CompletionItem(func.name, vscode.CompletionItemKind.Function);
        item.detail = func.desc;
        item.insertText = new vscode.SnippetString(`${func.name}(\${1:${func.params}})`);
        item.documentation = new vscode.MarkdownString(`**${func.name}(${func.params})**\n\n${func.desc}`);
        items.push(item);
    }
    
    return items;
}

function getNetworkCompletions(): vscode.CompletionItem[] {
    const items: vscode.CompletionItem[] = [];
    
    const networkFunctions = [
        { name: 'http_get', params: 'url, headers', desc: 'Perform HTTP GET request' },
        { name: 'http_post', params: 'url, data, headers', desc: 'Perform HTTP POST request' },
        { name: 'http_put', params: 'url, data, headers', desc: 'Perform HTTP PUT request' },
        { name: 'http_delete', params: 'url, headers', desc: 'Perform HTTP DELETE request' },
        { name: 'http_patch', params: 'url, data, headers', desc: 'Perform HTTP PATCH request' },
        { name: 'http_head', params: 'url, headers', desc: 'Perform HTTP HEAD request' },
        { name: 'http_options', params: 'url, headers', desc: 'Perform HTTP OPTIONS request' },
        { name: 'json_encode', params: 'value', desc: 'Encode value as JSON string' },
        { name: 'json_decode', params: 'json_string', desc: 'Decode JSON string to value' },
        { name: 'json_is_valid', params: 'json_string', desc: 'Check if string is valid JSON' },
        { name: 'url_parse', params: 'url', desc: 'Parse URL into components' },
        { name: 'url_build', params: 'components', desc: 'Build URL from components' },
        { name: 'url_encode', params: 'string', desc: 'URL encode string' },
        { name: 'url_decode', params: 'encoded_string', desc: 'URL decode string' },
        { name: 'base64_encode', params: 'data', desc: 'Encode data as base64' },
        { name: 'base64_decode', params: 'encoded_data', desc: 'Decode base64 data' },
        { name: 'websocket_connect', params: 'url, protocols', desc: 'Connect to WebSocket' },
        { name: 'websocket_send', params: 'socket, message', desc: 'Send WebSocket message' },
        { name: 'websocket_receive', params: 'socket', desc: 'Receive WebSocket message' },
        { name: 'websocket_close', params: 'socket', desc: 'Close WebSocket connection' },
        { name: 'form_encode', params: 'data', desc: 'Encode form data' },
        { name: 'form_decode', params: 'encoded_data', desc: 'Decode form data' }
    ];
    
    for (const func of networkFunctions) {
        const item = new vscode.CompletionItem(func.name, vscode.CompletionItemKind.Function);
        item.detail = func.desc;
        item.insertText = new vscode.SnippetString(`${func.name}(\${1:${func.params}})`);
        item.documentation = new vscode.MarkdownString(`**${func.name}(${func.params})**\n\n${func.desc}`);
        items.push(item);
    }
    
    return items;
}

function getDateTimeCompletions(): vscode.CompletionItem[] {
    const items: vscode.CompletionItem[] = [];
    
    const dateTimeFunctions = [
        { name: 'now', params: '', desc: 'Get current timestamp' },
        { name: 'current_time', params: '', desc: 'Get current time as struct' },
        { name: 'timestamp_to_struct', params: 'timestamp', desc: 'Convert timestamp to date struct' },
        { name: 'struct_to_timestamp', params: 'date_struct', desc: 'Convert date struct to timestamp' },
        { name: 'format_datetime', params: 'timestamp, format', desc: 'Format timestamp as string' },
        { name: 'parse_datetime', params: 'date_string, format', desc: 'Parse date string to timestamp' },
        { name: 'add_seconds', params: 'timestamp, seconds', desc: 'Add seconds to timestamp' },
        { name: 'add_minutes', params: 'timestamp, minutes', desc: 'Add minutes to timestamp' },
        { name: 'add_hours', params: 'timestamp, hours', desc: 'Add hours to timestamp' },
        { name: 'add_days', params: 'timestamp, days', desc: 'Add days to timestamp' },
        { name: 'add_weeks', params: 'timestamp, weeks', desc: 'Add weeks to timestamp' },
        { name: 'add_months', params: 'timestamp, months', desc: 'Add months to timestamp' },
        { name: 'add_years', params: 'timestamp, years', desc: 'Add years to timestamp' },
        { name: 'diff_seconds', params: 'timestamp1, timestamp2', desc: 'Difference in seconds' },
        { name: 'diff_minutes', params: 'timestamp1, timestamp2', desc: 'Difference in minutes' },
        { name: 'diff_hours', params: 'timestamp1, timestamp2', desc: 'Difference in hours' },
        { name: 'diff_days', params: 'timestamp1, timestamp2', desc: 'Difference in days' },
        { name: 'is_leap_year', params: 'year', desc: 'Check if year is leap year' },
        { name: 'days_in_month', params: 'year, month', desc: 'Get days in month' },
        { name: 'days_in_year', params: 'year', desc: 'Get days in year' },
        { name: 'day_of_week', params: 'timestamp', desc: 'Get day of week (0=Sunday)' },
        { name: 'day_of_year', params: 'timestamp', desc: 'Get day of year (1-366)' },
        { name: 'week_of_year', params: 'timestamp', desc: 'Get week of year' },
        { name: 'start_of_day', params: 'timestamp', desc: 'Get start of day timestamp' },
        { name: 'end_of_day', params: 'timestamp', desc: 'Get end of day timestamp' },
        { name: 'start_of_week', params: 'timestamp', desc: 'Get start of week timestamp' },
        { name: 'end_of_week', params: 'timestamp', desc: 'Get end of week timestamp' },
        { name: 'start_of_month', params: 'timestamp', desc: 'Get start of month timestamp' },
        { name: 'end_of_month', params: 'timestamp', desc: 'Get end of month timestamp' },
        { name: 'start_of_year', params: 'timestamp', desc: 'Get start of year timestamp' },
        { name: 'end_of_year', params: 'timestamp', desc: 'Get end of year timestamp' }
    ];
    
    for (const func of dateTimeFunctions) {
        const item = new vscode.CompletionItem(func.name, vscode.CompletionItemKind.Function);
        item.detail = func.desc;
        item.insertText = new vscode.SnippetString(`${func.name}(\${1:${func.params}})`);
        item.documentation = new vscode.MarkdownString(`**${func.name}(${func.params})**\n\n${func.desc}`);
        items.push(item);
    }
    
    return items;
}

// ===== HELPER FUNCTIONS FOR INTELLISENSE =====

function getHoverInfo(word: string): string | undefined {
    // Core functions hover info
    const coreInfo: { [key: string]: string } = {
        'is_int': '**is_int(value)** - Check if value is integer\n\nReturns `true` if the value is an integer, `false` otherwise.',
        'is_float': '**is_float(value)** - Check if value is float\n\nReturns `true` if the value is a floating-point number, `false` otherwise.',
        'is_string': '**is_string(value)** - Check if value is string\n\nReturns `true` if the value is a string, `false` otherwise.',
        'is_bool': '**is_bool(value)** - Check if value is boolean\n\nReturns `true` if the value is a boolean, `false` otherwise.',
        'is_array': '**is_array(value)** - Check if value is array\n\nReturns `true` if the value is an array, `false` otherwise.',
        'is_none': '**is_none(value)** - Check if value is None\n\nReturns `true` if the value is None/null, `false` otherwise.',
        'type_of': '**type_of(value)** - Get type name as string\n\nReturns a string representing the type of the value.',
        'to_int': '**to_int(value)** - Convert value to integer\n\nConverts the given value to an integer. Strings are parsed, floats are truncated.',
        'to_float': '**to_float(value)** - Convert value to float\n\nConverts the given value to a floating-point number.',
        'to_string': '**to_string(value)** - Convert value to string\n\nConverts the given value to its string representation.',
        'to_bool': '**to_bool(value)** - Convert value to boolean\n\nConverts the given value to a boolean using truthiness rules.',
        'len': '**len(value)** - Get length of array or string\n\nReturns the number of elements in an array or characters in a string.',
        'print': '**print(message)** - Print message to stdout\n\nPrints the given message to standard output.',
        'deep_copy': '**deep_copy(value)** - Create deep copy of value\n\nCreates a deep copy of the given value, recursively copying nested structures.'
    };
    
    // Math functions hover info
    const mathInfo: { [key: string]: string } = {
        'PI': '**PI** - Mathematical constant π (3.14159...)\n\nThe ratio of a circle\'s circumference to its diameter.',
        'E': '**E** - Mathematical constant e (2.71828...)\n\nEuler\'s number, the base of natural logarithms.',
        'PHI': '**PHI** - Golden ratio φ (1.61803...)\n\nThe golden ratio, often found in nature and art.',
        'TAU': '**TAU** - Mathematical constant τ = 2π\n\nThe ratio of a circle\'s circumference to its radius.',
        'abs': '**abs(x)** - Absolute value\n\nReturns the absolute value of x (distance from zero).',
        'sqrt': '**sqrt(x)** - Square root\n\nReturns the square root of x using Newton\'s method.',
        'sin': '**sin(x)** - Sine function\n\nReturns the sine of x (x in radians) using Taylor series.',
        'cos': '**cos(x)** - Cosine function\n\nReturns the cosine of x (x in radians) using Taylor series.',
        'pow': '**pow(base, exponent)** - Power function\n\nReturns base raised to the power of exponent.',
        'factorial': '**factorial(n)** - Factorial function\n\nReturns n! (n factorial). Only works for non-negative integers.',
        'now': '**now()** - Get current timestamp\n\nReturns the current Unix timestamp as an integer.',
        'current_time': '**current_time()** - Get current time as struct\n\nReturns a struct with year, month, day, hour, minute, second fields.',
        'timestamp_to_struct': '**timestamp_to_struct(timestamp)** - Convert timestamp to date struct\n\nConverts a Unix timestamp to a date/time struct with individual fields.'
    };
    
    return coreInfo[word] || mathInfo[word];
}

function getFunctionSignature(functionName: string): vscode.SignatureInformation | undefined {
    const signatures: { [key: string]: { label: string, params: string[], docs: string } } = {
        'is_int': { label: 'is_int(value)', params: ['value'], docs: 'Check if value is integer' },
        'is_float': { label: 'is_float(value)', params: ['value'], docs: 'Check if value is float' },
        'is_string': { label: 'is_string(value)', params: ['value'], docs: 'Check if value is string' },
        'is_bool': { label: 'is_bool(value)', params: ['value'], docs: 'Check if value is boolean' },
        'is_array': { label: 'is_array(value)', params: ['value'], docs: 'Check if value is array' },
        'is_none': { label: 'is_none(value)', params: ['value'], docs: 'Check if value is None' },
        'type_of': { label: 'type_of(value)', params: ['value'], docs: 'Get type name as string' },
        'to_int': { label: 'to_int(value)', params: ['value'], docs: 'Convert value to integer' },
        'to_float': { label: 'to_float(value)', params: ['value'], docs: 'Convert value to float' },
        'to_string': { label: 'to_string(value)', params: ['value'], docs: 'Convert value to string' },
        'to_bool': { label: 'to_bool(value)', params: ['value'], docs: 'Convert value to boolean' },
        'len': { label: 'len(value)', params: ['value'], docs: 'Get length of array or string' },
        'print': { label: 'print(message)', params: ['message'], docs: 'Print message to stdout' },
        'deep_copy': { label: 'deep_copy(value)', params: ['value'], docs: 'Create deep copy of value' },
        'abs': { label: 'abs(x)', params: ['x'], docs: 'Absolute value' },
        'sqrt': { label: 'sqrt(x)', params: ['x'], docs: 'Square root' },
        'sin': { label: 'sin(x)', params: ['x'], docs: 'Sine function' },
        'cos': { label: 'cos(x)', params: ['x'], docs: 'Cosine function' },
        'pow': { label: 'pow(base, exponent)', params: ['base', 'exponent'], docs: 'Power function' },
        'min': { label: 'min(a, b)', params: ['a', 'b'], docs: 'Minimum of two values' },
        'max': { label: 'max(a, b)', params: ['a', 'b'], docs: 'Maximum of two values' },
        'clamp': { label: 'clamp(value, min_val, max_val)', params: ['value', 'min_val', 'max_val'], docs: 'Clamp value between bounds' },
        'factorial': { label: 'factorial(n)', params: ['n'], docs: 'Factorial function' },
        'binomial': { label: 'binomial(n, k)', params: ['n', 'k'], docs: 'Binomial coefficient' },
        'now': { label: 'now()', params: [], docs: 'Get current timestamp' },
        'current_time': { label: 'current_time()', params: [], docs: 'Get current time as struct' },
        'timestamp_to_struct': { label: 'timestamp_to_struct(timestamp)', params: ['timestamp'], docs: 'Convert timestamp to date struct' },
        'http_get': { label: 'http_get(url, headers)', params: ['url', 'headers'], docs: 'Perform HTTP GET request' },
        'http_post': { label: 'http_post(url, data, headers)', params: ['url', 'data', 'headers'], docs: 'Perform HTTP POST request' },
        'json_encode': { label: 'json_encode(value)', params: ['value'], docs: 'Encode value as JSON string' },
        'json_decode': { label: 'json_decode(json_string)', params: ['json_string'], docs: 'Decode JSON string to value' },
        'read_file': { label: 'read_file(filename)', params: ['filename'], docs: 'Read entire file as string' },
        'write_file': { label: 'write_file(filename, content)', params: ['filename', 'content'], docs: 'Write string to file' },
        'array_push': { label: 'array_push(array, element)', params: ['array', 'element'], docs: 'Add element to end of array' },
        'array_pop': { label: 'array_pop(array)', params: ['array'], docs: 'Remove and return last element' },
        'string_length': { label: 'string_length(str)', params: ['str'], docs: 'Get string length' },
        'string_concat': { label: 'string_concat(str1, str2)', params: ['str1', 'str2'], docs: 'Concatenate two strings' }
    };
    
    const sig = signatures[functionName];
    if (!sig) return undefined;
    
    const signature = new vscode.SignatureInformation(sig.label, sig.docs);
    signature.parameters = sig.params.map(param => new vscode.ParameterInformation(param));
    
    return signature;
}

function getActiveParameter(beforeCursor: string): number {
    const commaCount = (beforeCursor.match(/,/g) || []).length;
    return commaCount;
}

function getStdlibLocation(word: string): vscode.Location | undefined {
    if (!vscode.workspace.workspaceFolders) return undefined;
    
    const workspaceRoot = vscode.workspace.workspaceFolders[0].uri.fsPath;
    const stdlibPath = path.join(workspaceRoot, 'stdlib');
    
    // Map function names to their stdlib files
    const functionToFile: { [key: string]: string } = {
        // Math functions
        'abs': 'math.vey', 'sqrt': 'math.vey', 'sin': 'math.vey', 'cos': 'math.vey', 'pow': 'math.vey',
        'min': 'math.vey', 'max': 'math.vey', 'factorial': 'math.vey', 'PI': 'math.vey', 'E': 'math.vey',
        
        // String functions
        'string_length': 'collections.vey', 'string_concat': 'collections.vey', 'string_substring': 'collections.vey',
        
        // Array functions
        'array_push': 'collections.vey', 'array_pop': 'collections.vey', 'array_sort': 'collections.vey',
        
        // IO functions
        'read_file': 'io.vey', 'write_file': 'io.vey', 'file_exists': 'io.vey',
        
        // Network functions
        'http_get': 'net.vey', 'http_post': 'net.vey', 'json_encode': 'net.vey', 'json_decode': 'net.vey',
        
        // DateTime functions
        'now': 'datetime.vey', 'current_time': 'datetime.vey', 'timestamp_to_struct': 'datetime.vey'
    };
    
    const fileName = functionToFile[word];
    if (!fileName) return undefined;
    
    const filePath = path.join(stdlibPath, fileName);
    if (!fs.existsSync(filePath)) return undefined;
    
    // For now, just return the file location. In a full implementation,
    // we would parse the file to find the exact line number of the function
    const uri = vscode.Uri.file(filePath);
    return new vscode.Location(uri, new vscode.Position(0, 0));
}