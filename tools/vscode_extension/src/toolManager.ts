import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { promisify } from 'util';
import { exec } from 'child_process';

const execAsync = promisify(exec);

export interface ToolInfo {
    name: string;
    executableName: string;
    description: string;
    found: boolean;
    path?: string;
}

export class ToolManager {
    private outputChannel: vscode.OutputChannel;
    
    constructor(outputChannel: vscode.OutputChannel) {
        this.outputChannel = outputChannel;
    }

    /**
     * Find a Veyra tool (fmt, lint, pkg) in various locations
     */
    findTool(toolName: string): string | null {
        const execName = process.platform === 'win32' ? `${toolName}.exe` : toolName;
        const searchPaths: string[] = [];

        // 1. Check workspace tools/target/release
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (workspaceFolder) {
            searchPaths.push(
                path.join(workspaceFolder.uri.fsPath, 'tools', 'target', 'release', execName),
                path.join(workspaceFolder.uri.fsPath, 'tools', 'target', 'debug', execName)
            );
        }

        // 2. Check configured path
        const config = vscode.workspace.getConfiguration('veyra');
        const configuredPath = config.get<string>(`${toolName}Path`);
        if (configuredPath) {
            searchPaths.push(configuredPath);
        }

        // 3. Check PATH environment
        searchPaths.push(toolName);

        // Search for the tool
        for (const toolPath of searchPaths) {
            if (this.isExecutableAvailable(toolPath)) {
                return toolPath;
            }
        }

        return null;
    }

    /**
     * Check if an executable is available
     */
    isExecutableAvailable(executablePath: string): boolean {
        try {
            // If it's just a name (no path), it might be in PATH
            if (!path.isAbsolute(executablePath) && !executablePath.includes(path.sep)) {
                return true; // Will be tested when actually run
            }
            
            return fs.existsSync(executablePath) && fs.statSync(executablePath).isFile();
        } catch {
            return false;
        }
    }

    /**
     * Get information about all Veyra tools
     */
    getAllToolsInfo(): ToolInfo[] {
        const tools = [
            { name: 'Formatter', executableName: 'veyra-fmt', description: 'Code formatter' },
            { name: 'Linter', executableName: 'veyra-lint', description: 'Static analyzer' },
            { name: 'Package Manager', executableName: 'veyra-pkg', description: 'Project manager' }
        ];

        return tools.map(tool => {
            const toolPath = this.findTool(tool.executableName);
            return {
                ...tool,
                found: toolPath !== null,
                path: toolPath || undefined
            };
        });
    }

    /**
     * Check if all required tools are available
     */
    async checkToolsAvailability(): Promise<boolean> {
        const tools = this.getAllToolsInfo();
        const missingTools = tools.filter(t => !t.found);

        if (missingTools.length > 0) {
            this.outputChannel.appendLine('[Warning] Some Veyra tools are not available:');
            for (const tool of missingTools) {
                this.outputChannel.appendLine(`  - ${tool.name} (${tool.executableName})`);
            }
            this.outputChannel.appendLine('');
            this.outputChannel.appendLine('To build the tools, run: cargo build --release in the tools directory');
            return false;
        }

        return true;
    }

    /**
     * Prompt user to build tools if they're missing
     */
    async promptToBuildTools(toolName: string): Promise<boolean> {
        const choice = await vscode.window.showWarningMessage(
            `${toolName} not found. Would you like to build the Veyra tools?`,
            { modal: false },
            'Build Tools',
            'Configure Path',
            'Continue Anyway'
        );

        if (choice === 'Build Tools') {
            await vscode.commands.executeCommand('veyra.buildTools');
            return true;
        } else if (choice === 'Configure Path') {
            await vscode.commands.executeCommand('workbench.action.openSettings', `veyra.${toolName}Path`);
            return false;
        }

        return false;
    }

    /**
     * Build all Veyra tools using Cargo
     */
    async buildTools(progressCallback?: (message: string, increment: number) => void): Promise<boolean> {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('No workspace folder open');
            return false;
        }

        const toolsPath = path.join(workspaceFolder.uri.fsPath, 'tools');
        if (!fs.existsSync(toolsPath)) {
            vscode.window.showErrorMessage('Tools directory not found. Open the Veyra project folder.');
            return false;
        }

        this.outputChannel.show();
        this.outputChannel.appendLine('');
        this.outputChannel.appendLine('═══════════════════════════════════════════════════');
        this.outputChannel.appendLine('Building Veyra Tools');
        this.outputChannel.appendLine('═══════════════════════════════════════════════════');
        this.outputChannel.appendLine('');
        this.outputChannel.appendLine('[Info] Starting build process...');
        this.outputChannel.appendLine(`[Info] Tools directory: ${toolsPath}`);
        this.outputChannel.appendLine('');

        try {
            if (progressCallback) progressCallback('Checking Rust installation...', 10);
            
            // Check if cargo is available
            try {
                await execAsync('cargo --version');
                this.outputChannel.appendLine('[OK] Cargo found');
            } catch {
                vscode.window.showErrorMessage(
                    'Cargo not found. Please install Rust: https://rustup.rs/',
                    'Open Rust Website'
                ).then(selection => {
                    if (selection === 'Open Rust Website') {
                        vscode.env.openExternal(vscode.Uri.parse('https://rustup.rs/'));
                    }
                });
                return false;
            }

            if (progressCallback) progressCallback('Building tools with Cargo...', 20);
            
            // Build in release mode
            this.outputChannel.appendLine('[Info] Running: cargo build --release');
            this.outputChannel.appendLine('[Info] This may take a few minutes...');
            this.outputChannel.appendLine('');

            const { stdout, stderr } = await execAsync('cargo build --release', {
                cwd: toolsPath,
                timeout: 600000 // 10 minutes
            });

            if (stdout) {
                this.outputChannel.appendLine(stdout);
            }
            if (stderr) {
                this.outputChannel.appendLine(stderr);
            }

            if (progressCallback) progressCallback('Verifying build...', 90);

            // Verify the tools were built
            const builtTools = this.getAllToolsInfo();
            const stillMissing = builtTools.filter(t => !t.found);

            if (stillMissing.length === 0) {
                this.outputChannel.appendLine('');
                this.outputChannel.appendLine('[OK] All tools built successfully!');
                this.outputChannel.appendLine('');
                this.outputChannel.appendLine('Available tools:');
                for (const tool of builtTools) {
                    this.outputChannel.appendLine(`  [OK] ${tool.name}: ${tool.path}`);
                }
                this.outputChannel.appendLine('');
                this.outputChannel.appendLine('═══════════════════════════════════════════════════');
                
                if (progressCallback) progressCallback('Build complete!', 100);
                
                vscode.window.showInformationMessage('Veyra tools built successfully!');
                return true;
            } else {
                this.outputChannel.appendLine('');
                this.outputChannel.appendLine('[Warning] Build completed but some tools are still missing:');
                for (const tool of stillMissing) {
                    this.outputChannel.appendLine(`  [X] ${tool.name}`);
                }
                vscode.window.showWarningMessage('Some tools were not built successfully. Check the output for details.');
                return false;
            }

        } catch (error: any) {
            this.outputChannel.appendLine('');
            this.outputChannel.appendLine('[Error] Build failed:');
            this.outputChannel.appendLine(error.message);
            if (error.stderr) {
                this.outputChannel.appendLine('');
                this.outputChannel.appendLine('Build errors:');
                this.outputChannel.appendLine(error.stderr);
            }
            this.outputChannel.appendLine('');
            this.outputChannel.appendLine('═══════════════════════════════════════════════════');

            vscode.window.showErrorMessage(
                'Failed to build Veyra tools. Check the output for details.',
                'Show Output'
            ).then(selection => {
                if (selection === 'Show Output') {
                    this.outputChannel.show();
                }
            });

            return false;
        }
    }

    /**
     * Show tool status in a quick pick menu
     */
    async showToolStatus(): Promise<void> {
        const tools = this.getAllToolsInfo();
        
        const items = tools.map(tool => ({
            label: tool.found ? `$(check) ${tool.name}` : `$(x) ${tool.name}`,
            description: tool.found ? tool.path : 'Not found',
            detail: tool.description,
            tool
        }));

        items.push({
            label: '$(tools) Build All Tools',
            description: 'Compile all Veyra tools',
            detail: 'Run cargo build --release',
            tool: { name: 'build', executableName: 'build', description: '', found: false }
        });

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Veyra Tools Status',
            matchOnDescription: true
        });

        if (selected && selected.tool.name === 'build') {
            await vscode.commands.executeCommand('veyra.buildTools');
        }
    }
}
