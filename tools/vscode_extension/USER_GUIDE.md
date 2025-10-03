# 🎨 Veyra VS Code Extension - User Guide

## Welcome to the Enhanced Veyra Experience!

This guide shows you how to use all the new UI features in the Veyra VS Code extension.

---

## 🚀 Quick Start (30 seconds)

1. **Open a `.vey` file** (or create one)
2. **Look at the editor toolbar** - You'll see the ▶️ Run button
3. **Click it** - Your code runs instantly!
4. **Check the status bar** at the bottom - See your file status and errors

That's it! You're ready to code in Veyra.

---

## 📍 Understanding the Interface

### 1. **Editor Toolbar** (Top of editor, when a `.vey` file is open)

```
┌─────────────────────────────────────────────────┐
│  filename.vey  ×    [▶️]  [🎨]                 │
└─────────────────────────────────────────────────┘
                     ↑     ↑
                     │     └── Quick Actions Menu
                     └── Run Veyra File
```

- **▶️ Run Button**: Execute your code immediately
- **🎨 Quick Actions**: Access all Veyra commands

### 2. **Status Bar** (Bottom of window)

```
┌─────────────────────────────────────────────────────────────┐
│  $(code-oss) Veyra: myfile.vey    $(error) 2 $(warning) 3  │
└─────────────────────────────────────────────────────────────┘
   ↑                                  ↑
   └── Click for Quick Actions       └── Click to view problems
```

**Left Side:**
- Shows current Veyra file name
- Click to open Quick Actions menu

**Right Side (when errors exist):**
- Shows error and warning counts
- Red background for errors
- Yellow background for warnings only
- Click to open Problems panel

**During Execution:**
```
│  $(sync~spin) Running...  │  ← Animated spinner while code runs
```

### 3. **Output Channel** (View → Output → Veyra)

Beautiful formatted logs:
```
═══════════════════════════════════════════════════
🎨 Veyra Language Extension
═══════════════════════════════════════════════════

✅ Veyra compiler found: C:\path\to\veyra.exe

═══════════════════════════════════════════════════
▶️  Running Veyra File
═══════════════════════════════════════════════════
💾 Saving file...
📄 File: myfile.vey
🔧 Compiler: C:\path\to\veyra.exe

✅ Execution started in terminal
═══════════════════════════════════════════════════
```

---

## 🎮 Using Commands

### Method 1: Quick Actions Menu (Recommended)

1. **Click the status bar** (where it says "Veyra: filename")
2. **Or** click the 🎨 button in the editor toolbar
3. **Choose an action** from the menu:

```
🎨 Select a Veyra action
┌───────────────────────────────────────────────────────┐
│ $(play) Run Veyra File                                │
│   Execute the current file                            │
├───────────────────────────────────────────────────────┤
│ $(gear) Build Project                                 │
│   Build the entire project                            │
├───────────────────────────────────────────────────────┤
│ $(pencil) Format File                                 │
│   Auto-format the current file                        │
├───────────────────────────────────────────────────────┤
│ $(check) Lint File                                    │
│   Check for code quality issues                       │
├───────────────────────────────────────────────────────┤
│ $(new-folder) New Project                             │
│   Create a new Veyra project                          │
├───────────────────────────────────────────────────────┤
│ $(output) Show Output                                 │
│   View Veyra output channel                           │
├───────────────────────────────────────────────────────┤
│ $(book) Documentation                                 │
│   Open Veyra documentation                            │
└───────────────────────────────────────────────────────┘
```

### Method 2: Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Run File | `Ctrl+F5` |
| Format File | `Shift+Alt+F` |
| Command Palette | `Ctrl+Shift+P` |
| Go to Definition | `F12` |
| IntelliSense | `Ctrl+Space` |
| Signature Help | `Ctrl+Shift+Space` |

### Method 3: Right-Click Menu

1. **Right-click** anywhere in a `.vey` file
2. **Select** from the context menu:
   - Run Veyra File
   - Format File
   - Lint File

### Method 4: Command Palette

1. **Press** `Ctrl+Shift+P`
2. **Type** "Veyra"
3. **Choose** from available commands

---

## 🎯 Common Tasks

### Running Your Code

**Option 1: One-Click Run**
1. Click the ▶️ button in the toolbar
2. Watch the terminal open automatically
3. See results immediately

**Option 2: Keyboard**
1. Press `Ctrl+F5`
2. Done!

**What You'll See:**
- Output channel logs the operation
- Status bar shows "Running..." briefly
- Terminal opens with: `🚀 Veyra: myfile.vey`
- Your program output appears

### Viewing Errors and Warnings

**Real-Time Display:**
- Errors appear as you type (red squiggles)
- Status bar shows counts: `$(error) 2 $(warning) 3`

**To View Details:**
1. Click the error counter in status bar
2. **Or** press `Ctrl+Shift+M`
3. Problems panel opens with full details

**Error Display:**
```
Problems (3 errors, 5 warnings)
  ❌ test.vey:15:10 - Unexpected token '{'
  ❌ test.vey:23:5  - Undefined variable 'x'
  ⚠️ test.vey:42:1  - Unused variable 'y'
```

### Formatting Your Code

**Auto-Format:**
1. Save the file (if `formatOnSave` is enabled)
2. **Or** press `Shift+Alt+F`
3. **Or** right-click → Format Document

**What Happens:**
- Progress notification appears
- Terminal shows formatting operation
- File reloads with formatted code
- Success message: "✨ myfile.vey formatted successfully!"

### Creating a New Project

1. **Open Quick Actions** (click status bar or 🎨 button)
2. **Select** "📦 New Project"
3. **Enter** project name (validates input)
4. **Choose** location (folder picker)
5. **Wait** for creation (progress shown)
6. **Choose** to open the project or stay

**Validation:**
- Project name must not be empty
- Only letters, numbers, hyphens, and underscores allowed
- Clear error messages if invalid

---

## 🎨 Visual Features Explained

### Status Bar Colors

**Green** (default): Everything is fine
```
│ $(code-oss) Veyra: myfile.vey │
```

**Red** background: Errors present
```
│ $(error) 3 $(warning) 5 │  ← Red background
```

**Yellow** background: Only warnings
```
│ $(warning) 2 │  ← Yellow background
```

### Terminal Branding

Each operation gets its own branded terminal:

```
🚀 Veyra: myfile.vey        (Running code)
🔨 Veyra Build               (Building project)
✨ Veyra Format              (Formatting code)
🔍 Veyra Lint                (Linting code)
📦 Veyra New Project         (Creating project)
```

### Progress Notifications

Long operations show progress:
```
┌──────────────────────────────────┐
│  ⚙️ Building Veyra project...    │
│  [████████████░░░░░░░░░] 60%     │
└──────────────────────────────────┘
```

---

## 💡 Tips & Tricks

### Tip 1: Use the Status Bar
The status bar is your command center. Click it for quick access to all Veyra features!

### Tip 2: Watch the Output Channel
Keep the output channel open (View → Output → Veyra) to see detailed logs of operations.

### Tip 3: Keyboard Shortcuts
Learn `Ctrl+F5` for running and `Shift+Alt+F` for formatting - they're huge time savers!

### Tip 4: IntelliSense
Just start typing and press `Ctrl+Space` to see all available functions with documentation.

### Tip 5: Error Navigation
Press `F8` to jump to the next error, `Shift+F8` for previous error.

### Tip 6: Hover for Help
Hover over any function to see its documentation, parameters, and usage examples.

### Tip 7: Right-Click
Right-click in any `.vey` file to access quick actions contextually.

---

## 🔧 Troubleshooting UI Issues

### "Run button doesn't appear"
- Make sure you have a `.vey` file open
- Check that the file is saved (not untitled)
- Reload window: `Ctrl+Shift+P` → "Reload Window"

### "Status bar not showing"
- Status bar only appears for `.vey` files
- Make sure the file has the `.vey` extension
- Check View → Appearance → Status Bar is enabled

### "Quick actions menu is empty"
- Ensure the extension is activated
- Check the output channel for errors
- Try reloading the window

### "Output channel not visible"
- Go to View → Output
- Select "Veyra" from the dropdown

---

## 🎓 Learning Path

### Beginner (Day 1)
1. ✅ Learn to run code with ▶️ button
2. ✅ Understand status bar indicators
3. ✅ Use IntelliSense for function suggestions

### Intermediate (Week 1)
1. ✅ Master keyboard shortcuts
2. ✅ Use the quick actions menu
3. ✅ Format and lint your code
4. ✅ Navigate errors efficiently

### Advanced (Month 1)
1. ✅ Customize settings for your workflow
2. ✅ Create and manage projects
3. ✅ Use go-to-definition and symbol navigation
4. ✅ Integrate with build systems

---

## 📚 Additional Resources

- **README.md**: Comprehensive feature list
- **UI_IMPROVEMENTS.md**: Technical details of UI enhancements
- **Output Channel**: Real-time logs and feedback
- **Problems Panel**: Detailed error information
- **IntelliSense**: Built-in documentation

---

## 🎉 Enjoy Coding in Veyra!

You now have a powerful, modern development environment for Veyra. The UI is designed to:

✅ Get out of your way when you're coding
✅ Provide help exactly when you need it
✅ Give clear, actionable feedback
✅ Make common tasks one-click easy
✅ Look beautiful and professional

**Happy coding!** 🚀

---

*Have feedback? Found a UI issue? Let us know through GitHub Issues!*
